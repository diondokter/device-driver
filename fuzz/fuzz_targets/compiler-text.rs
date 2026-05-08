#![no_main]

use device_driver_core::Target;
use libfuzzer_sys::fuzz_target;

// Goal: Don't have panics or ICE's
fuzz_target!(|source: &str| {
    if let Err(e) = device_driver_core::compile(source, Target::Rust, Target::Rust.get_compile_options()) {
        println!("======================\n{e:#}\n======================");
        panic!();
    }
});
