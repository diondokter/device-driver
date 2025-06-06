use std::{fs::DirEntry, path::PathBuf};

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=cases");
    println!("cargo::rerun-if-changed=../device-driver-cli");

    let test_cases = std::fs::read_dir("cases").unwrap();

    let mut test_cases_module = String::new();

    for test_case in test_cases {
        let test_case = test_case.unwrap();
        test_cases_module += &format!("#[cfg(test)]\n{}\n", generate_test_function(test_case));
    }

    std::fs::write(
        PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("test_cases.rs"),
        test_cases_module,
    )
    .unwrap();
}

fn generate_test_function(test_dir: DirEntry) -> String {
    let input = std::fs::read_to_string(test_dir.path().join("input.yaml")).unwrap();
    let output = std::fs::read_to_string(test_dir.path().join("output.rs")).unwrap();
    let test_name = test_dir.file_name().to_string_lossy().to_string();

    format!(
        "
#[test]
fn {test_name}() {{
    const INPUT: &str = {input:?};
    const OUTPUT: &str = {output:?};


    crate::run_test(INPUT, OUTPUT);
}}"
    )
}
