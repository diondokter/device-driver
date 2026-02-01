use std::{fs::DirEntry, path::PathBuf};

use itertools::Itertools;

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

    let inputs: Vec<PathBuf> = std::fs::read_dir(&test_dir_absolute)
        .unwrap()
        .filter(|entry| {
            entry
                .as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("input.")
        })
        .map(|entry| entry.unwrap().path())
        .collect();

    let test_name = test_dir.file_name().to_string_lossy().to_string();

    let ignore_tag = if test_name.ends_with('_') {
        "#[ignore = \"Test case ignored because case name ends with `_`\"]\n"
    } else {
        ""
    };

    let input_paths = inputs
        .iter()
        .map(|input_path| format!("Path::new(r\"{}\")", input_path.display()))
        .join(", ");
    let output_path = test_dir_absolute
        .join(format!("{test_name}.rs"))
        .display()
        .to_string();

    format!(
        "
#[test]
{ignore_tag}
fn {test_name}() {{
    crate::run_test(&[{input_paths}], &Path::new(r\"{output_path}\"));
}}"
    )
}
