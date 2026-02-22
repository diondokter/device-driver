use device_driver_diagnostics::Metadata;
use wasm_bindgen::prelude::*;

extern crate wasm_bindgen;

#[wasm_bindgen]
pub fn compile(source: &str, chars_per_line: usize) -> Output {
    let (output, diagnostics) = device_driver_core::compile(source);

    let mut diagnostics_string = String::new();
    diagnostics
        .print_to_fmt(
            &mut diagnostics_string,
            Metadata {
                source,
                source_path: "input.ddsl",
                term_width: Some(chars_per_line),
                ansi: true,
                unicode: true,
                anonymized_line_numbers: false,
            },
        )
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
