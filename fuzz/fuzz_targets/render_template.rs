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

fuzz_target!(|val: (&str, Vec<(&str, &str)>, Value)| {
    let (data, extra, value) = val;
    let mut tpl = tinytemplate::TinyTemplate::new();

    if tpl.add_template("template", data).is_err() {
        return;
    }

    for (name, data) in extra {
        let _ = tpl.add_template(name, data).is_err();
    }

    let _ = tpl.render("template", &value);
});
