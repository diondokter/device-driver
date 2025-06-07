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
    let test_dir_absolute = std::path::absolute(test_dir.path()).unwrap();

    let input_path = test_dir_absolute.join("input.yaml").display().to_string();
    let output_path = test_dir_absolute.join("output.rs").display().to_string();

    let test_name = test_dir.file_name().to_string_lossy().to_string();

    format!(
        "
#[test]
fn {test_name}() {{
    const INPUT: &str = {input_path:?};
    const OUTPUT: &str = {output_path:?};

    crate::run_test(&Path::new(INPUT), &Path::new(OUTPUT));
}}"
    )
}
