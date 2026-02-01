use std::path::PathBuf;

use wasm_bindgen::prelude::*;

extern crate wasm_bindgen;

#[wasm_bindgen]
pub fn compile(input: &str, chars_per_line: usize) -> Output {
    device_driver_core::reporting::set_miette_hook(false);

    let (output, diagnostics) =
        device_driver_core::transform_kdl(input, None, &PathBuf::from("input.kdl"));

    let mut diagnostics_string = String::new();
    diagnostics
        .print_to_fmt(&mut diagnostics_string, Some(chars_per_line))
        .unwrap();

    Output {
        code: output,
        diagnostics: diagnostics_string,
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct Output {
    pub code: String,
    pub diagnostics: String,
}
