use compiler::TemplateCompiler;
use error::Error::*;
use error::*;
use format;
use instruction::{Instruction, PathSlice};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Write;
use std::slice;
use Formatter;

enum ContextElement<'render, 'template> {
    Object(&'render Value),
    Named(&'template str, &'render Value),
    Iteration(
        &'template str,
        &'render Value,
        usize,
        slice::Iter<'render, Value>,
    ),
}

struct RenderContext<'render, 'template> {
    context_stack: Vec<ContextElement<'render, 'template>>,
}
impl<'render, 'template> RenderContext<'render, 'template> {
    fn lookup(&self, path: PathSlice) -> Result<&'render Value> {
        for stack_layer in self.context_stack.iter().rev() {
            match stack_layer {
                ContextElement::Object(obj) => return self.lookup_in(path, obj),
                ContextElement::Named(name, obj) => {
                    if *name == path[0] {
                        return self.lookup_in(&path[1..], obj);
                    }
                }
                ContextElement::Iteration(name, obj, _, _) => {
                    if *name == path[0] {
                        return self.lookup_in(&path[1..], obj);
                    }
                }
            }
        }
        panic!("Attempted to do a lookup with an empty context stack. That shouldn't be possible.")
    }

    fn lookup_in(&self, path: PathSlice, object: &'render Value) -> Result<&'render Value> {
        let mut current = object;
        for step in path.iter() {
            match current.get(step) {
                Some(next) => current = next,
                None => return Err(lookup_error(step, path, current)),
            }
        }
        Ok(current)
    }

    fn lookup_index(&self) -> Result<usize> {
        for stack_layer in self.context_stack.iter().rev() {
            match stack_layer {
                ContextElement::Iteration(_, _, index, _) => return Ok(*index),
                _ => continue,
            }
        }
        Err(RenderError {
            msg: "Used @index outside of a foreach block.".to_string(),
        })
    }
}

pub(crate) struct Template<'template> {
    instructions: Vec<Instruction<'template>>,
    template_len: usize,
}
impl<'template> Template<'template> {
    pub fn compile(text: &'template str) -> Result<Template> {
        Ok(Template {
            template_len: text.len(),
            instructions: TemplateCompiler::new(text).compile()?,
        })
    }

    pub fn render(
        &self,
        context: &Value,
        template_registry: &HashMap<&str, Template>,
        formatter_registry: &HashMap<&str, Box<Formatter>>,
    ) -> Result<String> {
        // The length of the original template seems like a reasonable guess at the length of the
        // output.
        let mut output = String::with_capacity(self.template_len);
        self.render_into(context, template_registry, formatter_registry, &mut output)?;
        Ok(output)
    }

    pub fn render_into(
        &self,
        context: &Value,
        template_registry: &HashMap<&str, Template>,
        formatter_registry: &HashMap<&str, Box<Formatter>>,
        output: &mut String,
    ) -> Result<()> {
        let mut program_counter = 0;
        let mut render_context = RenderContext {
            context_stack: vec![ContextElement::Object(context)],
        };

        while program_counter < self.instructions.len() {
            match &self.instructions[program_counter] {
                Instruction::Literal(text) => {
                    output.push_str(text);
                    program_counter += 1;
                }
                Instruction::Value(path) => {
                    let first = *path.first().unwrap();
                    if first.starts_with('@') {
                        match first {
                            "@index" => {
                                write!(output, "{}", render_context.lookup_index()?).unwrap()
                            }
                            _ => panic!(), // This should have been caught by the parser.
                        }
                    } else {
                        let value_to_render = render_context.lookup(path)?;
                        format(value_to_render, output)?;
                    }
                    program_counter += 1;
                }
                Instruction::FormattedValue(path, name) => {
                    let value_to_render = render_context.lookup(path)?;
                    match formatter_registry.get(name) {
                        Some(formatter) => formatter(value_to_render, output)?,
                        None => return Err(unknown_formatter(name)),
                    }
                    program_counter += 1;
                }
                Instruction::Branch(path, target) => {
                    let value_to_render = render_context.lookup(path)?;
                    let mut truthy = match value_to_render {
                        Value::Null => true,
                        Value::Bool(b) => !*b,
                        Value::Number(n) => match n.as_f64() {
                            Some(float) => float != 0.0,
                            None => {
                                return Err(truthiness_error(path));
                            }
                        },
                        Value::String(s) => !s.is_empty(),
                        Value::Array(arr) => !arr.is_empty(),
                        Value::Object(_) => {
                            return Err(truthiness_error(path));
                        }
                    };

                    if truthy {
                        program_counter = *target;
                    } else {
                        program_counter += 1;
                    }
                }
                Instruction::PushContext(path) => {
                    let context_value = render_context.lookup(path)?;
                    render_context
                        .context_stack
                        .push(ContextElement::Object(context_value));
                    program_counter += 1;
                }
                Instruction::PushNamedContext(path, name) => {
                    let context_value = render_context.lookup(path)?;
                    render_context
                        .context_stack
                        .push(ContextElement::Named(name, context_value));
                    program_counter += 1;
                }
                Instruction::PushIterationContext(path, name) => {
                    let context_value = render_context.lookup(path)?;
                    match context_value {
                        Value::Array(ref arr) => {
                            render_context.context_stack.push(ContextElement::Iteration(
                                name,
                                &Value::Null,
                                std::usize::MAX,
                                arr.iter(),
                            ))
                        }
                        _ => return Err(not_iterable_error(path)),
                    };
                    program_counter += 1;
                }
                Instruction::PopContext => {
                    render_context.context_stack.pop();
                    program_counter += 1;
                }
                Instruction::Goto(target) => {
                    program_counter = *target;
                }
                Instruction::Iterate(target) => {
                    match render_context.context_stack.last_mut() {
                        Some(ContextElement::Iteration(_, val, index, iter)) => match iter.next() {
                            Some(new_val) => {
                                *val = new_val;
                                *index = index.wrapping_add(1);
                                program_counter += 1;
                            }
                            None => {
                                program_counter = *target;
                            }
                        },
                        _ => panic!("Malformed program."),
                    };
                }
                Instruction::Call(template_name, path) => {
                    let context_value = render_context.lookup(path)?;
                    match template_registry.get(template_name) {
                        Some(templ) => templ.render_into(
                            context_value,
                            template_registry,
                            formatter_registry,
                            output,
                        )?,
                        None => return Err(unknown_template(template_name)),
                    }
                    program_counter += 1;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use compiler::TemplateCompiler;

    fn compile(text: &'static str) -> Template<'static> {
        Template {
            template_len: text.len(),
            instructions: TemplateCompiler::new(text).compile().unwrap(),
        }
    }

    #[derive(Serialize)]
    struct NestedContext {
        value: usize,
    }

    #[derive(Serialize)]
    struct TestContext {
        number: usize,
        string: &'static str,
        boolean: bool,
        null: Option<usize>,
        array: Vec<usize>,
        nested: NestedContext,
    }

    fn context() -> Value {
        let ctx = TestContext {
            number: 5,
            string: "test",
            boolean: true,
            null: None,
            array: vec![1, 2, 3],
            nested: NestedContext { value: 10 },
        };
        serde_json::to_value(&ctx).unwrap()
    }

    fn other_templates() -> HashMap<&'static str, Template<'static>> {
        let mut map = HashMap::new();
        map.insert("my_macro", compile("{{value}}"));
        map
    }

    fn format(value: &Value, output: &mut String) -> Result<()> {
        output.push_str("{");
        ::format(value, output)?;
        output.push_str("}");
        Ok(())
    }

    fn formatters() -> HashMap<&'static str, Box<Formatter>> {
        let mut map = HashMap::<&'static str, Box<Formatter>>::new();
        map.insert("my_formatter", Box::new(format));
        map
    }

    #[test]
    fn test_literal() {
        let template = compile("Hello!");
        let context = context();
        let template_registry = other_templates();
        let formatter_registry = formatters();
        let string = template
            .render(&context, &template_registry, &formatter_registry)
            .unwrap();
        assert_eq!("Hello!", &string);
    }

    #[test]
    fn test_value() {
        let template = compile("{{ number }}");
        let context = context();
        let template_registry = other_templates();
        let formatter_registry = formatters();
        let string = template
            .render(&context, &template_registry, &formatter_registry)
            .unwrap();
        assert_eq!("5", &string);
    }

    #[test]
    fn test_path() {
        let template = compile("The number of the day is {{ nested.value }}.");
        let context = context();
        let template_registry = other_templates();
        let formatter_registry = formatters();
        let string = template
            .render(&context, &template_registry, &formatter_registry)
            .unwrap();
        assert_eq!("The number of the day is 10.", &string);
    }

    #[test]
    fn test_if_taken() {
        let template = compile("{% if boolean %}Hello!{% endif %}");
        let context = context();
        let template_registry = other_templates();
        let formatter_registry = formatters();
        let string = template
            .render(&context, &template_registry, &formatter_registry)
            .unwrap();
        assert_eq!("Hello!", &string);
    }

    #[test]
    fn test_if_not_taken() {
        let template = compile("{% if null %}Hello!{% endif %}");
        let context = context();
        let template_registry = other_templates();
        let formatter_registry = formatters();
        let string = template
            .render(&context, &template_registry, &formatter_registry)
            .unwrap();
        assert_eq!("", &string);
    }

    #[test]
    fn test_if_else_taken() {
        let template = compile("{% if boolean %}Hello!{% else %}Goodbye!{% endif %}");
        let context = context();
        let template_registry = other_templates();
        let formatter_registry = formatters();
        let string = template
            .render(&context, &template_registry, &formatter_registry)
            .unwrap();
        assert_eq!("Hello!", &string);
    }

    #[test]
    fn test_if_else_not_taken() {
        let template = compile("{% if null %}Hello!{% else %}Goodbye!{% endif %}");
        let context = context();
        let template_registry = other_templates();
        let formatter_registry = formatters();
        let string = template
            .render(&context, &template_registry, &formatter_registry)
            .unwrap();
        assert_eq!("Goodbye!", &string);
    }

    #[test]
    fn test_nested_ifs() {
        let template = compile(
            "{% if boolean %}Hi, {% if null %}there!{% else %}Hello!{% endif %}{% endif %}",
        );
        let context = context();
        let template_registry = other_templates();
        let formatter_registry = formatters();
        let string = template
            .render(&context, &template_registry, &formatter_registry)
            .unwrap();
        assert_eq!("Hi, Hello!", &string);
    }

    #[test]
    fn test_with() {
        let template = compile("{% with nested %}{{value}}{%endwith%}");
        let context = context();
        let template_registry = other_templates();
        let formatter_registry = formatters();
        let string = template
            .render(&context, &template_registry, &formatter_registry)
            .unwrap();
        assert_eq!("10", &string);
    }

    #[test]
    fn test_named_with() {
        let template = compile("{% with nested as n %}{{ n.value }} {{ number }}{%endwith%}");
        let context = context();
        let template_registry = other_templates();
        let formatter_registry = formatters();
        let string = template
            .render(&context, &template_registry, &formatter_registry)
            .unwrap();
        assert_eq!("10 5", &string);
    }

    #[test]
    fn test_for_loop() {
        let template = compile("{% for a in array %}{{ a }}{% endfor %}");
        let context = context();
        let template_registry = other_templates();
        let formatter_registry = formatters();
        let string = template
            .render(&context, &template_registry, &formatter_registry)
            .unwrap();
        assert_eq!("123", &string);
    }

    #[test]
    fn test_for_loop_index() {
        let template = compile("{% for a in array %}{{ @index }}{% endfor %}");
        let context = context();
        let template_registry = other_templates();
        let formatter_registry = formatters();
        let string = template
            .render(&context, &template_registry, &formatter_registry)
            .unwrap();
        assert_eq!("012", &string);
    }

    #[test]
    fn test_whitespace_stripping_value() {
        let template = compile("1  \n\t   {{- number -}}  \n   1");
        let context = context();
        let template_registry = other_templates();
        let formatter_registry = formatters();
        let string = template
            .render(&context, &template_registry, &formatter_registry)
            .unwrap();
        assert_eq!("151", &string);
    }

    #[test]
    fn test_call() {
        let template = compile("{% call my_macro with nested %}");
        let context = context();
        let template_registry = other_templates();
        let formatter_registry = formatters();
        let string = template
            .render(&context, &template_registry, &formatter_registry)
            .unwrap();
        assert_eq!("10", &string);
    }

    #[test]
    fn test_formatter() {
        let template = compile("{{ nested.value | my_formatter }}");
        let context = context();
        let template_registry = other_templates();
        let formatter_registry = formatters();
        let string = template
            .render(&context, &template_registry, &formatter_registry)
            .unwrap();
        assert_eq!("{10}", &string);
    }

    #[test]
    fn test_unknown() {
        let template = compile("{{ foobar }}");
        let context = context();
        let template_registry = other_templates();
        let formatter_registry = formatters();
        template
            .render(&context, &template_registry, &formatter_registry)
            .unwrap_err();
    }
}
