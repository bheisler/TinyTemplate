use error::Error::*;
use error::Result;
use instruction::{Branch, Instruction, Path};

const UNKNOWN: usize = std::usize::MAX;

enum Block {
    Branch(usize),
    With,
}

fn parse_path(text: &str) -> Path {
    text.split('.').collect::<Vec<_>>()
}

pub(crate) struct TemplateCompiler<'template> {
    remaining_text: &'template str,
    instructions: Vec<Instruction<'template>>,
    block_stack: Vec<Block>,
}
impl<'template> TemplateCompiler<'template> {
    pub fn new(text: &'template str) -> TemplateCompiler<'template> {
        TemplateCompiler {
            remaining_text: text,
            instructions: vec![],
            block_stack: vec![],
        }
    }

    pub fn compile(mut self) -> Result<Vec<Instruction<'template>>> {
        while !self.remaining_text.is_empty() {
            if self.remaining_text.starts_with("{{") {
                let path = self.consume_value()?;
                self.instructions.push(Instruction::Value(path));
            } else if self.remaining_text.starts_with("{%") {
                let (discriminant, rest) = self.consume_block()?;
                match discriminant {
                    "if" => {
                        let path = parse_path(rest);
                        self.block_stack
                            .push(Block::Branch(self.instructions.len()));
                        self.instructions.push(Instruction::Branch(Branch {
                            path,
                            invert: true,
                            target: UNKNOWN,
                        }));
                    }
                    "else" => {
                        self.expect_empty(rest)?;
                        let num_instructions = self.instructions.len() + 1;
                        let path_clone: Path<'template> =
                            self.with_unclosed_branch(|b| match b {
                                Instruction::Branch(branch) => {
                                    branch.target = num_instructions;
                                    Ok(branch.path.clone())
                                }
                                _ => panic!(),
                            })?;
                        self.block_stack
                            .push(Block::Branch(self.instructions.len()));
                        self.instructions.push(Instruction::Goto(UNKNOWN))
                    }
                    "endif" => {
                        self.expect_empty(rest)?;
                        let num_instructions = self.instructions.len();
                        self.with_unclosed_branch(|b| match b {
                            Instruction::Branch(branch) => {
                                branch.target = num_instructions;
                                Ok(())
                            }
                            Instruction::Goto(target) => {
                                *target = num_instructions;
                                Ok(())
                            }
                            _ => panic!(),
                        })?;
                    }
                    "with" => {
                        let (path, name) = self.parse_with(rest);
                        let instructions = match name {
                            Some(name) => Instruction::PushNamedContext(path, name),
                            None => Instruction::PushContext(path),
                        };
                        self.instructions.push(instructions);
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
                let text = self.consume_text();
                self.instructions.push(Instruction::Literal(text));
            }
        }
        Ok(self.instructions)
    }

    fn expect_empty(&self, text: &str) -> Result<()> {
        if text == "" {
            Ok(())
        } else {
            Err(ParseError {
                msg: format!("Unexpected text '{}'", text),
            })
        }
    }

    fn with_unclosed_branch<F, T>(&mut self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Instruction<'template>) -> Result<T>,
    {
        let branch_block = self.block_stack.pop();
        if let Some(Block::Branch(index)) = branch_block {
            f(&mut self.instructions[index])
        } else {
            Err(ParseError {
                msg: "Found a closing endif or else which doesn't match with a preceding if."
                    .to_string(),
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

    fn consume_value(&mut self) -> Result<Path<'template>> {
        let tag = self.consume_tag("}}")?;
        let path = &tag[2..(tag.len() - 2)].trim();
        Ok(parse_path(path))
    }

    fn consume_block(&mut self) -> Result<(&'template str, &'template str)> {
        let tag = self.consume_tag("%}")?;
        let block = &tag[2..(tag.len() - 2)].trim();
        let discriminant = block.split_whitespace().next().unwrap_or(block);
        let rest = &block[discriminant.len()..].trim();
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

    fn parse_with(&self, with_text: &'template str) -> (Path<'template>, Option<&'template str>) {
        if let Some(index) = with_text.find(" as ") {
            let (path_str, name_str) = with_text.split_at(index);
            let path = parse_path(path_str.trim());
            let name = name_str[" as ".len()..].trim();
            (path, Some(name))
        } else {
            let path = parse_path(with_text);
            (path, None)
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(needless_pass_by_value)]

    use super::*;
    use instruction::Instruction::*;

    fn compile(text: &'static str) -> Result<Vec<Instruction<'static>>> {
        TemplateCompiler::new(text).compile()
    }

    fn branch(path: Path<'static>, invert: bool, target: usize) -> Instruction<'static> {
        Instruction::Branch(::instruction::Branch {
            path,
            invert,
            target,
        })
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
        assert_eq!(&branch(vec!["foo"], true, 2), &instructions[0]);
        assert_eq!(&Literal("Hello!"), &instructions[1]);
    }

    #[test]
    fn test_if_else_endif() {
        let text = "{% if foo %}Hello!{% else %}Goodbye!{% endif %}";
        let instructions = compile(text).unwrap();
        assert_eq!(4, instructions.len());
        assert_eq!(&branch(vec!["foo"], true, 3), &instructions[0]);
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
    fn test_unclosed_tags() {
        let tags = vec![
            "{{",
            "{{ foo.bar",
            "{{ foo.bar\n%}",
            "{%",
            "{% if foo.bar",
            "{% if foo.bar \n%}",
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
}
