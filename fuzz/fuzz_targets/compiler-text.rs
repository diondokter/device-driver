#![no_main]

use device_driver_core::{CodegenTarget, CompileOptions, GeneralOptions, MirOptions};
use libfuzzer_sys::fuzz_target;

#[derive(Debug, arbitrary::Arbitrary)]
struct Input<'a> {
    source: &'a str,
    seed: u64,
}

// Goal: Don't have panics or ICE's
fuzz_target!(|input: Input<'_>| {
    match device_driver_core::compile(
        input.source,
        CompileOptions {
            general_options: GeneralOptions::default(),
            mir_options: MirOptions {
                randomize_mir_passes: true,
                randomize_mir_passes_seed: Some(input.seed),
                check_assumptions: true,
            },
            target: CodegenTarget::Rust(Default::default()),
        },
    ) {
        Ok((_output, diagnostics)) => {
            let mut d = String::new();
            diagnostics
                .print_to_fmt(
                    &mut d,
                    device_driver_core::Metadata {
                        source: input.source,
                        source_path: "fuzz.ddsl",
                        term_width: None,
                        ansi: false,
                        unicode: false,
                        anonymized_line_numbers: false,
                    },
                )
                .unwrap();
        }
        Err(e) => {
            println!("======================\n{e:#}\n======================");
            panic!();
        }
    }
});
