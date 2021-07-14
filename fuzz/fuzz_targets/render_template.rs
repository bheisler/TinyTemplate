#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    let mut tpl = tinytemplate::TinyTemplate::new();

    if tpl.add_template("template", data).is_err() {
        return;
    }

    let _ = tinytemplate::TinyTemplate::new().render("template", &());
});
