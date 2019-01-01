use compiler::TemplateCompiler;
use error::Error::*;
use error::{Error, Result};
use instruction::{Branch, Instruction, Path};
use serde::Serialize;
use serde_json::{self, Value};
use std::fmt::Write;

fn lookup_error(step: &str, path: &Path, current: &Value) -> Error {
    let avail_str = if let Value::Object(object_map) = current {
        let mut avail_str = " Available values at this level are '".to_string();
        for (i, key) in object_map.keys().enumerate() {
            avail_str.push_str(key);
            if i > 0 {
                avail_str.push_str(", ");
            }
        }
        avail_str
    } else {
        "".to_string()
    };

    RenderError {
        msg: format!(
            "Failed to find value '{}' from path '{}'.{}",
            step,
            path_to_str(path),
            avail_str
        ),
    }
}

fn truthiness_error(path: &Path) -> Error {
    RenderError {
        msg: format!(
            "Path '{}' produced a value which could not be checked for truthiness.",
            path_to_str(path)
        ),
    }
}

fn unprintable_error(path: &Path) -> Error {
    RenderError {
        msg: format!(
            "Expected a printable value for path '{}' but found array or object.",
            path_to_str(path)
        ),
    }
}

fn path_to_str(path: &Path) -> String {
    let mut path_str = "".to_string();
    for (i, step) in path.iter().enumerate() {
        path_str.push_str(step);
        if i > 0 {
            path_str.push('.');
        }
    }
    path_str
}

struct RenderContext<'render> {
    context_stack: Vec<&'render Value>,
}
impl<'render> RenderContext<'render> {
    pub fn lookup(&self, path: &Path) -> Result<&'render Value> {
        let mut current = self.context_stack[self.context_stack.len() - 1];
        for step in path.iter() {
            match current.get(step) {
                Some(next) => current = next,
                None => return Err(lookup_error(step, path, current)),
            }
        }
        Ok(current)
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

    pub fn render(&self, context: &Value) -> Result<String> {
        // The length of the original template seems like a reasonable guess at the length of the
        // output.
        let mut output = String::with_capacity(self.template_len);
        let mut program_counter = 0;
        let mut render_context = RenderContext {
            context_stack: vec![context],
        };

        while program_counter < self.instructions.len() {
            match &self.instructions[program_counter] {
                Instruction::Literal(text) => {
                    output.push_str(text);
                    program_counter += 1;
                }
                Instruction::Value(path) => {
                    let value_to_render = render_context.lookup(path)?;
                    match value_to_render {
                        Value::Null => {}
                        Value::Bool(b) => write!(output, "{}", b).unwrap(),
                        Value::Number(n) => write!(output, "{}", n).unwrap(),
                        Value::String(s) => output.push_str(s),
                        _ => return Err(unprintable_error(path)),
                    };
                    program_counter += 1;
                }
                Instruction::Branch(branch) => {
                    let Branch {
                        path,
                        invert,
                        target,
                    } = branch;

                    let value_to_render = render_context.lookup(path)?;
                    let mut truthy = match value_to_render {
                        Value::Null => false,
                        Value::Bool(b) => *b,
                        Value::Number(n) => match n.as_f64() {
                            Some(float) => float == 0.0,
                            None => {
                                return Err(truthiness_error(path));
                            }
                        },
                        Value::String(s) => s == "",
                        Value::Array(arr) => arr.is_empty(),
                        Value::Object(_) => {
                            return Err(truthiness_error(path));
                        }
                    };

                    if *invert {
                        truthy = !truthy;
                    }

                    if truthy {
                        program_counter = *target;
                    } else {
                        program_counter += 1;
                    }
                }
                Instruction::PushContext(path) => {
                    let context_value = render_context.lookup(path)?;
                    render_context.context_stack.push(context_value);
                    program_counter += 1;
                }
                Instruction::PopContext => {
                    render_context.context_stack.pop();
                    program_counter += 1;
                }
            }
        }

        Ok(output)
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

    #[test]
    fn test_literal() {
        let template = compile("Hello!");
        let context = context();
        let string = template.render(&context).unwrap();
        assert_eq!("Hello!", &string);
    }

    #[test]
    fn test_value() {
        let template = compile("{{ number }}");
        let context = context();
        let string = template.render(&context).unwrap();
        assert_eq!("5", &string);
    }

    #[test]
    fn test_path() {
        let template = compile("The number of the day is {{ nested.value }}.");
        let context = context();
        let string = template.render(&context).unwrap();
        assert_eq!("The number of the day is 10.", &string);
    }

    #[test]
    fn test_if_taken() {
        let template = compile("{% if boolean %}Hello!{% endif %}");
        let context = context();
        let string = template.render(&context).unwrap();
        assert_eq!("Hello!", &string);
    }

    #[test]
    fn test_if_not_taken() {
        let template = compile("{% if null %}Hello!{% endif %}");
        let context = context();
        let string = template.render(&context).unwrap();
        assert_eq!("", &string);
    }

    #[test]
    fn test_if_else_taken() {
        let template = compile("{% if boolean %}Hello!{% else %}Goodbye!{% endif %}");
        let context = context();
        let string = template.render(&context).unwrap();
        assert_eq!("Hello!", &string);
    }

    #[test]
    fn test_if_else_not_taken() {
        let template = compile("{% if null %}Hello!{% else %}Goodbye!{% endif %}");
        let context = context();
        let string = template.render(&context).unwrap();
        assert_eq!("Goodbye!", &string);
    }

    #[test]
    fn test_nested_ifs() {
        let template = compile(
            "{% if boolean %}Hi, {% if null %}there!{% else %}Hello!{% endif %}{% endif %}",
        );
        let context = context();
        let string = template.render(&context).unwrap();
        assert_eq!("Hi, Hello!", &string);
    }

    #[test]
    fn test_with() {
        let template = compile("{% with nested %}{{value}}{%endwith%}");
        let context = context();
        let string = template.render(&context).unwrap();
        assert_eq!("10", &string);
    }

    #[test]
    fn test_unknown() {
        let template = compile("{{ foobar }}");
        let context = context();
        template.render(&context).unwrap_err();
    }
}
