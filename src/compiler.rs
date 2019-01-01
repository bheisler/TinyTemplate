use error::Error::*;
use error::Result;
use instruction::{Instruction, Path};

const UNKNOWN: usize = std::usize::MAX;

enum Block {
    Branch(usize),
    For(usize),
    With,
}

fn parse_path(text: &str) -> Result<Path> {
    if !text.starts_with('@') {
        Ok(text.split('.').collect::<Vec<_>>())
    } else if text == "@index" {
        Ok(vec![text])
    } else {
        Err(ParseError {
            msg: format!("Invalid keyword name '{}'", text),
        })
    }
}

pub(crate) struct TemplateCompiler<'template> {
    remaining_text: &'template str,
    instructions: Vec<Instruction<'template>>,
    block_stack: Vec<Block>,
    trim_next: bool,
}
impl<'template> TemplateCompiler<'template> {
    pub fn new(text: &'template str) -> TemplateCompiler<'template> {
        TemplateCompiler {
            remaining_text: text,
            instructions: vec![],
            block_stack: vec![],
            trim_next: false,
        }
    }

    pub fn compile(mut self) -> Result<Vec<Instruction<'template>>> {
        while !self.remaining_text.is_empty() {
            if self.remaining_text.starts_with("{{") {
                let (path, name) = self.consume_value()?;
                let instruction = match name {
                    Some(name) => Instruction::FormattedValue(path, name),
                    None => Instruction::Value(path),
                };
                self.instructions.push(instruction);
            } else if self.remaining_text.starts_with("{#") {
                let tag = self.consume_tag("#}")?;
                let comment = tag[2..(tag.len() - 2)].trim();
                if comment.starts_with('-') {
                    self.trim_last_whitespace();
                }
                if comment.ends_with('-') {
                    self.trim_next_whitespace();
                }
            } else if self.remaining_text.starts_with("{%") {
                let (discriminant, rest) = self.consume_block()?;
                match discriminant {
                    "if" => {
                        let path = parse_path(rest)?;
                        self.block_stack
                            .push(Block::Branch(self.instructions.len()));
                        self.instructions.push(Instruction::Branch(path, UNKNOWN));
                    }
                    "else" => {
                        self.expect_empty(rest)?;
                        let num_instructions = self.instructions.len() + 1;
                        self.close_branch(num_instructions)?;
                        self.block_stack
                            .push(Block::Branch(self.instructions.len()));
                        self.instructions.push(Instruction::Goto(UNKNOWN))
                    }
                    "endif" => {
                        self.expect_empty(rest)?;
                        let num_instructions = self.instructions.len();
                        self.close_branch(num_instructions)?;
                    }
                    "with" => {
                        let (path, name) = self.parse_with(rest)?;
                        let instruction = match name {
                            Some(name) => Instruction::PushNamedContext(path, name),
                            None => Instruction::PushContext(path),
                        };
                        self.instructions.push(instruction);
                        self.block_stack.push(Block::With);
                    }
                    "endwith" => {
                        self.expect_empty(rest)?;
                        if let Some(Block::With) = self.block_stack.pop() {
                            self.instructions.push(Instruction::PopContext)
                        } else {
                            return Err(ParseError {
                                msg: "Found a closing endwith that doesn't match with a preceeding with.".to_string()
                            });
                        }
                    }
                    "for" => {
                        let (path, name) = self.parse_for(rest)?;
                        self.instructions
                            .push(Instruction::PushIterationContext(path, name));
                        self.block_stack.push(Block::For(self.instructions.len()));
                        self.instructions.push(Instruction::Iterate(UNKNOWN));
                    }
                    "endfor" => {
                        self.expect_empty(rest)?;
                        let num_instructions = self.instructions.len() + 1;
                        let goto_target = self.close_for(num_instructions)?;
                        self.instructions.push(Instruction::Goto(goto_target));
                        self.instructions.push(Instruction::PopContext);
                    }
                    "call" => {
                        let (name, path) = self.parse_call(rest)?;
                        self.instructions.push(Instruction::Call(name, path));
                    }
                    _ => {
                        return Err(ParseError {
                            msg: format!("Unknown block type '{}'", discriminant),
                        })
                    }
                }
            } else if self.remaining_text.starts_with('{') {
                return Err(ParseError {
                    msg: "Unexpected '{'".to_string(),
                });
            } else {
                let mut text = self.consume_text();
                if self.trim_next {
                    text = text.trim_start();
                    self.trim_next = false;
                }
                self.instructions.push(Instruction::Literal(text));
            }
        }
        Ok(self.instructions)
    }

    fn expect_empty(&self, text: &str) -> Result<()> {
        if text.is_empty() {
            Ok(())
        } else {
            Err(ParseError {
                msg: format!("Unexpected text '{}'", text),
            })
        }
    }

    fn close_branch(&mut self, new_target: usize) -> Result<()> {
        let branch_block = self.block_stack.pop();
        if let Some(Block::Branch(index)) = branch_block {
            match &mut self.instructions[index] {
                Instruction::Branch(_, target) => {
                    *target = new_target;
                    Ok(())
                }
                Instruction::Goto(target) => {
                    *target = new_target;
                    Ok(())
                }
                _ => panic!(),
            }
        } else {
            Err(ParseError {
                msg: "Found a closing endif or else which doesn't match with a preceding if."
                    .to_string(),
            })
        }
    }

    fn close_for(&mut self, new_target: usize) -> Result<usize> {
        let branch_block = self.block_stack.pop();
        if let Some(Block::For(index)) = branch_block {
            match &mut self.instructions[index] {
                Instruction::Iterate(target) => {
                    *target = new_target;
                    Ok(index)
                }
                _ => panic!(),
            }
        } else {
            Err(ParseError {
                msg: "Found a closing endfor which doesn't match with a preceding for.".to_string(),
            })
        }
    }

    fn consume_text(&mut self) -> &'template str {
        let position = self
            .remaining_text
            .find('{')
            .unwrap_or_else(|| self.remaining_text.len());
        let (text, remaining) = self.remaining_text.split_at(position);
        self.remaining_text = remaining;
        text
    }

    fn consume_value(&mut self) -> Result<(Path<'template>, Option<&'template str>)> {
        let tag = self.consume_tag("}}")?;
        let mut tag = tag[2..(tag.len() - 2)].trim();
        if tag.starts_with('-') {
            tag = tag[1..].trim();
            self.trim_last_whitespace();
        }
        if tag.ends_with('-') {
            tag = tag[0..tag.len() - 1].trim();
            self.trim_next_whitespace();
        }

        if let Some(index) = tag.find('|') {
            let (path_str, name_str) = tag.split_at(index);
            let name = name_str[1..].trim();
            let path = parse_path(path_str.trim())?;
            Ok((path, Some(name)))
        } else {
            Ok((parse_path(tag)?, None))
        }
    }

    fn trim_last_whitespace(&mut self) {
        if let Some(Instruction::Literal(text)) = self.instructions.last_mut() {
            *text = text.trim_end();
        }
    }

    fn trim_next_whitespace(&mut self) {
        self.trim_next = true;
    }

    fn consume_block(&mut self) -> Result<(&'template str, &'template str)> {
        let tag = self.consume_tag("%}")?;
        let mut block = tag[2..(tag.len() - 2)].trim();
        if block.starts_with('-') {
            block = block[1..].trim();
            self.trim_last_whitespace();
        }
        if block.ends_with('-') {
            block = block[0..block.len() - 1].trim();
            self.trim_next_whitespace();
        }
        let discriminant = block.split_whitespace().next().unwrap_or(block);
        let rest = block[discriminant.len()..].trim();
        Ok((discriminant, rest))
    }

    fn consume_tag(&mut self, expected_close: &str) -> Result<&'template str> {
        if let Some(line) = self.remaining_text.lines().next() {
            if let Some(pos) = line.find(expected_close) {
                let (tag, remaining) = self.remaining_text.split_at(pos + expected_close.len());
                self.remaining_text = remaining;
                Ok(tag)
            } else {
                Err(ParseError {
                    msg: format!(
                        "Expected a closing '{}' but found end-of-line instead.",
                        expected_close
                    ),
                })
            }
        } else {
            Err(ParseError {
                msg: format!(
                    "Expected a closing '{}' but found end-of-text instead.",
                    expected_close
                ),
            })
        }
    }

    fn parse_with(
        &self,
        with_text: &'template str,
    ) -> Result<(Path<'template>, Option<&'template str>)> {
        if let Some(index) = with_text.find(" as ") {
            let (path_str, name_str) = with_text.split_at(index);
            let path = parse_path(path_str.trim())?;
            let name = name_str[" as ".len()..].trim();
            Ok((path, Some(name)))
        } else {
            let path = parse_path(with_text)?;
            Ok((path, None))
        }
    }

    fn parse_for(&self, for_text: &'template str) -> Result<(Path<'template>, &'template str)> {
        if let Some(index) = for_text.find(" in ") {
            let (name_str, path_str) = for_text.split_at(index);
            let name = name_str.trim();
            let path = parse_path(path_str[" in ".len()..].trim())?;
            Ok((path, name))
        } else {
            Err(ParseError {
                msg: format!("Unable to parse for block text '{}'", for_text),
            })
        }
    }

    fn parse_call(&self, call_text: &'template str) -> Result<(&'template str, Path<'template>)> {
        if let Some(index) = call_text.find(" with ") {
            let (name_str, path_str) = call_text.split_at(index);
            let name = name_str.trim();
            let path = parse_path(path_str[" with ".len()..].trim())?;
            Ok((name, path))
        } else {
            Err(ParseError {
                msg: format!("Unable to parse call block text '{}'", call_text),
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use instruction::Instruction::*;

    fn compile(text: &'static str) -> Result<Vec<Instruction<'static>>> {
        TemplateCompiler::new(text).compile()
    }

    #[test]
    fn test_compile_literal() {
        let text = "Test String";
        let instructions = compile(text).unwrap();
        assert_eq!(1, instructions.len());
        assert_eq!(&Literal(text), &instructions[0]);
    }

    #[test]
    fn test_compile_value() {
        let text = "{{ foobar }}";
        let instructions = compile(text).unwrap();
        assert_eq!(1, instructions.len());
        assert_eq!(&Value(vec!["foobar"]), &instructions[0]);
    }

    #[test]
    fn test_compile_value_with_formatter() {
        let text = "{{ foobar | my_formatter }}";
        let instructions = compile(text).unwrap();
        assert_eq!(1, instructions.len());
        assert_eq!(
            &FormattedValue(vec!["foobar"], "my_formatter"),
            &instructions[0]
        );
    }

    #[test]
    fn test_dotted_path() {
        let text = "{{ foo.bar }}";
        let instructions = compile(text).unwrap();
        assert_eq!(1, instructions.len());
        assert_eq!(&Value(vec!["foo", "bar"]), &instructions[0]);
    }

    #[test]
    fn test_mixture() {
        let text = "Hello {{ name }}, how are you?";
        let instructions = compile(text).unwrap();
        assert_eq!(3, instructions.len());
        assert_eq!(&Literal("Hello "), &instructions[0]);
        assert_eq!(&Value(vec!["name"]), &instructions[1]);
        assert_eq!(&Literal(", how are you?"), &instructions[2]);
    }

    #[test]
    fn test_if_endif() {
        let text = "{% if foo %}Hello!{% endif %}";
        let instructions = compile(text).unwrap();
        assert_eq!(2, instructions.len());
        assert_eq!(&Branch(vec!["foo"], 2), &instructions[0]);
        assert_eq!(&Literal("Hello!"), &instructions[1]);
    }

    #[test]
    fn test_if_else_endif() {
        let text = "{% if foo %}Hello!{% else %}Goodbye!{% endif %}";
        let instructions = compile(text).unwrap();
        assert_eq!(4, instructions.len());
        assert_eq!(&Branch(vec!["foo"], 3), &instructions[0]);
        assert_eq!(&Literal("Hello!"), &instructions[1]);
        assert_eq!(&Goto(4), &instructions[2]);
        assert_eq!(&Literal("Goodbye!"), &instructions[3]);
    }

    #[test]
    fn test_with() {
        let text = "{% with foo %}Hello!{% endwith %}";
        let instructions = compile(text).unwrap();
        assert_eq!(3, instructions.len());
        assert_eq!(&PushContext(vec!["foo"]), &instructions[0]);
        assert_eq!(&Literal("Hello!"), &instructions[1]);
        assert_eq!(&PopContext, &instructions[2]);
    }

    #[test]
    fn test_named_with() {
        let text = "{% with foo as bar %}Hello!{% endwith %}";
        let instructions = compile(text).unwrap();
        assert_eq!(3, instructions.len());
        assert_eq!(&PushNamedContext(vec!["foo"], "bar"), &instructions[0]);
        assert_eq!(&Literal("Hello!"), &instructions[1]);
        assert_eq!(&PopContext, &instructions[2]);
    }

    #[test]
    fn test_foreach() {
        let text = "{% for foo in bar.baz %}{{ foo }}{% endfor %}";
        let instructions = compile(text).unwrap();
        assert_eq!(5, instructions.len());
        assert_eq!(
            &PushIterationContext(vec!["bar", "baz"], "foo"),
            &instructions[0]
        );
        assert_eq!(&Iterate(4), &instructions[1]);
        assert_eq!(&Value(vec!["foo"]), &instructions[2]);
        assert_eq!(&Goto(1), &instructions[3]);
        assert_eq!(&PopContext, &instructions[4]);
    }

    #[test]
    fn test_strip_whitespace_value() {
        let text = "Hello,     {{- name -}}   , how are you?";
        let instructions = compile(text).unwrap();
        assert_eq!(3, instructions.len());
        assert_eq!(&Literal("Hello,"), &instructions[0]);
        assert_eq!(&Value(vec!["name"]), &instructions[1]);
        assert_eq!(&Literal(", how are you?"), &instructions[2]);
    }

    #[test]
    fn test_strip_whitespace_block() {
        let text = "Hello,     {%- if name -%}    {{name}}    {%- endif -%}   , how are you?";
        let instructions = compile(text).unwrap();
        assert_eq!(6, instructions.len());
        assert_eq!(&Literal("Hello,"), &instructions[0]);
        assert_eq!(&Branch(vec!["name"], 5), &instructions[1]);
        assert_eq!(&Literal(""), &instructions[2]);
        assert_eq!(&Value(vec!["name"]), &instructions[3]);
        assert_eq!(&Literal(""), &instructions[4]);
        assert_eq!(&Literal(", how are you?"), &instructions[5]);
    }

    #[test]
    fn test_comment() {
        let text = "Hello, {# foo bar baz #} there!";
        let instructions = compile(text).unwrap();
        assert_eq!(2, instructions.len());
        assert_eq!(&Literal("Hello, "), &instructions[0]);
        assert_eq!(&Literal(" there!"), &instructions[1]);
    }

    #[test]
    fn test_strip_whitespace_comment() {
        let text = "Hello, \t\n    {#- foo bar baz -#} \t  there!";
        let instructions = compile(text).unwrap();
        assert_eq!(2, instructions.len());
        assert_eq!(&Literal("Hello,"), &instructions[0]);
        assert_eq!(&Literal("there!"), &instructions[1]);
    }

    #[test]
    fn test_call() {
        let text = "{% call my_macro with foo.bar %}";
        let instructions = compile(text).unwrap();
        assert_eq!(1, instructions.len());
        assert_eq!(&Call("my_macro", vec!["foo", "bar"]), &instructions[0]);
    }

    #[test]
    fn test_unclosed_tags() {
        let tags = vec![
            "{{",
            "{{ foo.bar",
            "{{ foo.bar\n }}",
            "{%",
            "{% if foo.bar",
            "{% if foo.bar \n%}",
            "{#",
            "{# if foo.bar",
            "{# if foo.bar \n#}",
        ];
        for tag in tags {
            compile(tag).unwrap_err();
        }
    }

    #[test]
    fn test_mismatched_blocks() {
        let text = "{% if foo %}{% with bar %}{% endif %} {% endwith %}";
        compile(text).unwrap_err();
    }

    #[test]
    fn test_disallows_invalid_keywords() {
        let text = "{{ @foo }}";
        compile(text).unwrap_err();
    }
}
