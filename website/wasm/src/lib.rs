use std::path::PathBuf;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn compile(input: &str) -> Output {
    device_driver_compiler::reporting::set_miette_hook(false);

    let (output, diagnostics) =
        device_driver_compiler::transform_kdl(input, None, &PathBuf::from("input.kdl"));

    let mut diagnostics_string = String::new();
    diagnostics.print_to_fmt(&mut diagnostics_string).unwrap();

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
