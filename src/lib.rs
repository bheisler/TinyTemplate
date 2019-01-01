extern crate serde;
extern crate serde_json;

#[cfg(test)]
#[cfg_attr(test, macro_use)]
extern crate serde_derive;

mod compiler;
pub mod error;
mod instruction;
mod template;

use error::*;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Write;
use template::Template;

/*
TODO:
- Implement error detail handling by calculating the line/column when an error occurs
- HTML escaping?
- Benchmark
- Write documentation
- CI builds
- Build my own clone of serde_json::Value so I can drop serde_json?
*/
pub type Formatter = Fn(&Value, &mut String) -> Result<()>;

pub fn format(value: &Value, output: &mut String) -> Result<()> {
    match value {
        Value::Null => Ok(()),
        Value::Bool(b) => {
            write!(output, "{}", b)?;
            Ok(())
        }
        Value::Number(n) => {
            write!(output, "{}", n)?;
            Ok(())
        }
        Value::String(s) => {
            output.push_str(s);
            Ok(())
        }
        _ => Err(unprintable_error()),
    }
}

pub struct TinyTemplate<'template> {
    templates: HashMap<&'template str, Template<'template>>,
    formatters: HashMap<&'template str, Box<Formatter>>,
}
impl<'template> TinyTemplate<'template> {
    pub fn new() -> TinyTemplate<'template> {
        TinyTemplate {
            templates: HashMap::default(),
            formatters: HashMap::default(),
        }
    }

    pub fn add_template(&mut self, name: &'template str, text: &'template str) -> Result<()> {
        let template = Template::compile(text)?;
        self.templates.insert(name, template);
        Ok(())
    }

    pub fn add_formatter<F>(&mut self, name: &'template str, formatter: F)
    where
        F: 'static + Fn(&Value, &mut String) -> Result<()>,
    {
        self.formatters.insert(name, Box::new(formatter));
    }

    pub fn render<C>(&self, template: &str, context: &C) -> Result<String>
    where
        C: Serialize,
    {
        let value = serde_json::to_value(context)?;
        match self.templates.get(template) {
            Some(tmpl) => tmpl.render(&value, &self.templates, &self.formatters),
            None => Err(Error::UnknownTemplate {
                msg: format!("Unknown template '{}'", template),
            }),
        }
    }
}
impl<'template> Default for TinyTemplate<'template> {
    fn default() -> TinyTemplate<'template> {
        TinyTemplate::new()
    }
}
