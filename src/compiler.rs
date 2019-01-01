use instruction::{Instruction, Path};
use error::Result;
use error::Error::*;

pub(crate) struct TemplateCompiler<'template> {
    full_text: &'template str,
    remaining_text: &'template str
}
impl<'template> TemplateCompiler<'template> {
    pub fn new(text: &'template str) -> TemplateCompiler<'template> {
        TemplateCompiler {
            full_text: text,
            remaining_text: text
        }
    }

    pub fn compile(mut self) -> Result<Vec<Instruction<'template>>> {
        let mut instructions = vec![];
        while !self.remaining_text.is_empty() {
            if self.remaining_text.starts_with("{{") {
                let path = self.consume_value()?;
                instructions.push(Instruction::Value(path));
            }
            else {
                let text = self.consume_text();
                instructions.push(Instruction::Literal(text));
            }
        }
        Ok(instructions)
    }

    fn consume_text(&mut self) -> &'template str {
        let position = self.remaining_text.find('{').unwrap_or_else(|| self.remaining_text.len());
        let (text, remaining) = self.remaining_text.split_at(position);
        self.remaining_text = remaining;
        text
    }

    fn consume_value(&mut self) -> Result<Path<'template>> {
        let tag = self.consume_tag("}}")?;
        let path = &tag[2..(tag.len() - 2)].trim();
        let components = path.split(".").collect();
        Ok(components)
    }

    fn consume_tag(&mut self, expected_close: &str) -> Result<&'template str> {
        if let Some(line) = self.remaining_text.lines().next() {
            if let Some(pos) = line.find(expected_close) {
                let (tag, remaining) = self.remaining_text.split_at(pos + expected_close.len());
                self.remaining_text = remaining;
                Ok(tag)
            }
            else {
                Err(ParseError{
                    msg: format!("Expected a closing '{}' but found end-of-line instead.", expected_close)
                })
            }
        }
        else {
            Err(ParseError{
                msg: format!("Expected a closing '{}' but found end-of-text instead.", expected_close)
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn compile(text: &'static str) -> Result<Vec<Instruction<'static>>> {
        TemplateCompiler::new(text).compile()
    }

    fn assert_literal(expected: &str, instruction: &Instruction) {
        match instruction {
            Instruction::Literal(text) => {
                assert_eq!(&expected, text);
            },
            _ => panic!("Expected literal '{}' but was {:?}", expected, instruction),
        }
    }

    fn assert_value(expected_path: Vec<&str>, instruction: &Instruction) {
        match instruction {
            Instruction::Value(path) => {
                assert_eq!(&expected_path, path);
            },
            _ => panic!("Expected path '{}' but was {:?}", expected_path.join("."), instruction),
        }
    }

    #[test]
    fn test_compile_literal() {
        let text = "Test String";
        let instructions = compile(text).unwrap();
        assert_eq!(1, instructions.len());
        assert_literal(text, &instructions[0]);
    }

    #[test]
    fn test_compile_value() {
        let text = "{{ foobar }}";
        let instructions = compile(text).unwrap();
        assert_eq!(1, instructions.len());
        assert_value(vec!["foobar"], &instructions[0]);
    }

    #[test]
    fn test_dotted_path() {
        let text = "{{ foo.bar }}";
        let instructions = compile(text).unwrap();
        assert_eq!(1, instructions.len());
        assert_value(vec!["foo", "bar"], &instructions[0]);
    }

    #[test]
    fn test_mixture() {
        let text = "Hello {{ name }}, how are you?";
        let instructions = compile(text).unwrap();
        assert_eq!(3, instructions.len());
        assert_literal("Hello ", &instructions[0]);
        assert_value(vec!["name"], &instructions[1]);
        assert_literal(", how are you?", &instructions[2]);
    }

    #[test]
    fn test_unclosed_tags() {
        let tags = vec![
            "{{",
            "{{ foo.bar",
            "{{ foo.bar\n",
        ];
        for tag in tags {
            compile(tag).unwrap_err();
        }
    }
}