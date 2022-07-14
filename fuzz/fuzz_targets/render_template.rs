#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use serde::Serialize;

#[derive(Arbitrary, Debug, Serialize)]
enum Value {
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(std::collections::HashMap<String, Value>),
}

fuzz_target!(|val: (&str, Value)| {
    let (data, value) = val;
    let mut tpl = tinytemplate::TinyTemplate::new();

    if tpl.add_template("template", data).is_err() {
        return;
    }

    let _ = tinytemplate::TinyTemplate::new().render("template", &value);
});
