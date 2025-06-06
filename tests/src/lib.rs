include!(concat!(env!("OUT_DIR"), "/test_cases.rs"));

pub fn run_test(input: &str, output: &str) {
    let transformed = device_driver_generation::transform_yaml(input, "Device");
    pretty_assertions::assert_str_eq!(output, transformed);
}
