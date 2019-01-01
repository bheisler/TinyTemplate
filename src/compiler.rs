use error::Error::*;
use error::Result;
use instruction::{Instruction, Path};

const UNKNOWN: usize = std::usize::MAX;

fn parse_path(text: &str) -> Path {
    text.split('.').collect::<Vec<_>>()
}

pub(crate) struct TemplateCompiler<'template> {
    full_text: &'template str,
    remaining_text: &'template str,
    instructions: Vec<Instruction<'template>>,
}
impl<'template> TemplateCompiler<'template> {
    pub fn new(text: &'template str) -> TemplateCompiler<'template> {
        TemplateCompiler {
            full_text: text,
            remaining_text: text,
            instructions: vec![],
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
                        self.instructions.push(Instruction::Branch {
                            path,
                            invert: true,
                            taken: UNKNOWN,
                        })
                    }
                    "endif" => {
                        self.expect_empty(rest)?;
                        self.close_branch()?;
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

    fn close_branch(&mut self) -> Result<()> {
        let num_instructions = self.instructions.len();
        let instr = self.instructions.iter_mut().rfind(|instr| match instr {
            Instruction::Branch { taken, .. } if *taken == UNKNOWN => true,
            _ => false,
        });

        if let Some(Instruction::Branch { taken, .. }) = instr {
            *taken = num_instructions;
            Ok(())
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
}

#[cfg(test)]
mod test {
    #![allow(needless_pass_by_value)]

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
        assert_eq!(
            &Branch {
                path: vec!["foo"],
                invert: true,
                taken: 2
            },
            &instructions[0]
        );
        assert_eq!(&Literal("Hello!"), &instructions[1]);
    }

    #[test]
    fn test_unclosed_tags() {
        let tags = vec!["{{", "{{ foo.bar", "{{ foo.bar\n"];
        for tag in tags {
            compile(tag).unwrap_err();
        }
    }
}
