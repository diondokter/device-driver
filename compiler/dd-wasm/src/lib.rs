use clap::Parser;
use device_driver_core::{RustCodegenOptions, Target};
use device_driver_diagnostics::{Metadata, ResultExt};
use wasm_bindgen::prelude::*;

extern crate wasm_bindgen;

#[wasm_bindgen]
pub fn compile(source: &str, chars_per_line: usize, target: TargetArg, options: &str) -> Output {
    let options = options.replace("\r\n", " ").replace('\n', " ");
    let options = options.split(' ').filter(|s| !s.is_empty());

    let target = match target {
        TargetArg::Rust => {
            let codegen_options = match RustCodegenOptions::try_parse_from(options) {
                Ok(codegen_options) => codegen_options,
                Err(e) => {
                    return Output {
                        code: String::new(),
                        diagnostics: e.to_string(),
                    };
                }
            };

            Target::Rust(codegen_options)
        }
    };

    let (output, diagnostics_string) = match device_driver_core::compile(source, target)
        .with_message(|| "internal compiler error")
    {
        Ok((output, diagnostics)) => {
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
            (output, diagnostics_string)
        }
        Err(e) => (String::new(), e.to_report_string()),
    };

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

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub enum TargetArg {
    Rust,
}
