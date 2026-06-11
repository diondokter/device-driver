#![no_main]

use device_driver_core::Target;
use libfuzzer_sys::fuzz_target;

// Goal: Don't have panics or ICE's
fuzz_target!(|source: &str| {
    match device_driver_core::compile(source, Target::Rust(Default::default())) {
        Ok((_output, diagnostics)) => {
            let mut d = String::new();
            diagnostics
                .print_to_fmt(
                    &mut d,
                    device_driver_core::Metadata {
                        source,
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
