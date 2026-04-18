use device_driver_core::Target;
use device_driver_diagnostics::{Metadata, ResultExt};
use wasm_bindgen::prelude::*;

extern crate wasm_bindgen;

#[wasm_bindgen]
pub fn compile(source: &str, chars_per_line: usize, target: TargetArg, options: &str) -> Output {
    let mut compile_options = Target::from(target).get_compile_options();

    let options = options.replace("\r\n", " ").replace('\n', " ");
    let mut options = options.split(' ').filter(|s| !s.is_empty());

    while let Some(option) = options.next() {
        let value = if option == "-C" {
            options.next()
        } else if let Some(value) = option.strip_prefix("-C") {
            Some(value)
        } else {
            return Output {
                code: String::new(),
                diagnostics: format!("unknown compiler option: \"{option}\""),
            };
        };

        let Some(value) = value else {
            return Output {
                code: String::new(),
                diagnostics: "-C flag not followed up with a `<key>=<value>`".into(),
            };
        };

        let Some((key, value)) = value.split_once('=') else {
            return Output {
                code: String::new(),
                diagnostics: "-C flag not followed up with a `<key>=<value>`".into(),
            };
        };

        if !compile_options.add(key, value.into()) {
            if compile_options.possible_options().contains(&key) {
                return Output {
                    code: String::new(),
                    diagnostics: format!("duplicate key found: {key}"),
                };
            } else {
                return Output {
                    code: String::new(),
                    diagnostics: format!(
                        "key not recognized. Expected one of: {}",
                        compile_options.possible_options().join(", ")
                    ),
                };
            }
        }
    }

    let (output, diagnostics_string) =
        match device_driver_core::compile(source, Target::Rust, compile_options)
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

impl From<TargetArg> for Target {
    fn from(value: TargetArg) -> Self {
        match value {
            TargetArg::Rust => Self::Rust,
        }
    }
}
