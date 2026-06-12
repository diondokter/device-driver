use clap::Parser;
use device_driver_core::{CodegenTarget, CompileOptions, MirOptions, RustCodegenOptions};
use device_driver_diagnostics::{Metadata, ResultExt};
use wasm_bindgen::prelude::*;

extern crate wasm_bindgen;

#[derive(Parser, Debug, Clone)]
#[command(no_binary_name = true)]
struct RustCompileOptions {
    #[command(flatten)]
    pub mir_options: MirOptions,
    #[command(flatten)]
    pub rust_codegen_options: RustCodegenOptions,
}

impl From<RustCompileOptions> for CompileOptions {
    fn from(value: RustCompileOptions) -> Self {
        Self {
            mir_options: value.mir_options,
            target: CodegenTarget::Rust(value.rust_codegen_options),
        }
    }
}

#[wasm_bindgen]
pub fn compile(source: &str, chars_per_line: usize, target: TargetArg, options: &str) -> Output {
    let options = options.replace("\r\n", " ").replace('\n', " ");
    let options = options.split(' ').filter(|s| !s.is_empty());

    let compile_options = match target {
        TargetArg::Rust => match RustCompileOptions::try_parse_from(options) {
            Ok(codegen_options) => codegen_options,
            Err(e) => {
                return Output {
                    code: String::new(),
                    diagnostics: e.render().ansi().to_string(),
                };
            }
        }
        .into(),
    };

    let (output, diagnostics_string) = match device_driver_core::compile(source, compile_options)
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
