#![no_main]

use device_driver_codegen::Target;
use libfuzzer_sys::fuzz_target;

// Goal: Don't have panics or ICE's
fuzz_target!(|source: &str| {
    let _ = device_driver_core::compile(source, Target::Rust, Target::Rust.get_compile_options());
});
