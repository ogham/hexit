#![no_main]
use libfuzzer_sys::fuzz_target;
use std::str;
use hexit_lang::{Program, ConstantsTable};

fuzz_target!(|data: &[u8]| {
    let string = match str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return,
    };

    let _ = Program::read(string).map(|prog| {
        let constants = ConstantsTable::builtin_set();
        let _ = prog.run(&constants, None);
    });
});
