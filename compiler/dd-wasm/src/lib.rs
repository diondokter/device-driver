use clap::Parser;
use device_driver_core::{
    CodegenTarget, CompileOptions, DocsCodegenOptions, GeneralOptions, MirOptions,
    RustCodegenOptions,
};
use device_driver_diagnostics::{Metadata, ResultExt};
use wasm_bindgen::prelude::*;

extern crate wasm_bindgen;

#[derive(Parser, Debug, Clone)]
#[command(no_binary_name = true)]
struct RustCompileOptions {
    #[command(flatten)]
    pub general_options: GeneralOptions,
    #[command(flatten)]
    pub mir_options: MirOptions,
    #[command(flatten)]
    pub rust_codegen_options: RustCodegenOptions,
}

impl From<RustCompileOptions> for CompileOptions {
    fn from(value: RustCompileOptions) -> Self {
        Self {
            general_options: value.general_options,
            mir_options: value.mir_options,
            target: CodegenTarget::Rust(value.rust_codegen_options),
        }
    }
}

#[derive(Parser, Debug, Clone)]
#[command(no_binary_name = true)]
struct DocsCompileOptions {
    #[command(flatten)]
    pub general_options: GeneralOptions,
    #[command(flatten)]
    pub mir_options: MirOptions,
    #[command(flatten)]
    pub docs_codegen_options: DocsCodegenOptions,
}

impl From<DocsCompileOptions> for CompileOptions {
    fn from(value: DocsCompileOptions) -> Self {
        Self {
            general_options: value.general_options,
            mir_options: value.mir_options,
            target: CodegenTarget::Docs(value.docs_codegen_options),
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
        TargetArg::Docs => match DocsCompileOptions::try_parse_from(options) {
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
        Ok((output_files, diagnostics)) => {
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
            if let [output_file] = output_files.as_slice() {
                (output_file.contents.clone(), diagnostics_string)
            } else {
                // TODO: Support multiple files
                (String::new(), "did not get a single file result".into())
            }
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
    Docs,
}
