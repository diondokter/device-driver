use std::path::Path;

pub const OUTPUT_HEADER: &str = r#"---
[package]
edition = "2024"
[dependencies]
device-driver = { path="../../../device-driver" }
---
fn main() {}

"#;

include!(concat!(env!("OUT_DIR"), "/test_cases.rs"));

pub fn run_test(input_path: &Path, output_path: &Path) {
    let input = std::fs::read_to_string(input_path).unwrap();
    let output = std::fs::read_to_string(output_path).unwrap();

    let transformed =
        OUTPUT_HEADER.to_string() + &device_driver_generation::transform_yaml(&input, "Device");
    pretty_assertions::assert_str_eq!(output, transformed);

    let output_extension = output_path.extension().unwrap().to_str().unwrap();
    match output_extension {
        "rs" => {
            compile_output(output_path);
        }
        _ => unimplemented!(),
    }
}

fn compile_output(output_path: &Path) {
    let mut cmd = std::process::Command::new("cargo");

    cmd.arg("+nightly");
    cmd.arg("-Zscript");
    cmd.arg(output_path);

    let output = cmd.output().unwrap();

    if !output.status.success() {
        panic!("{}", String::from_utf8(output.stderr).unwrap());
    }
}
