use std::path::{Path, PathBuf};

use device_driver_core::{CodegenTarget, CompileOptions};
use device_driver_diagnostics::Metadata;
use device_driver_tests::get_compile_options;

fn main() {
    let args = std::env::args().skip(1);

    let mut accept_flag = false;
    let mut cases_path = "./cases".to_owned();

    for arg in args {
        match arg.as_str() {
            path if cases_path.is_empty() => cases_path = path.into(),
            "accept" => accept_flag = true,
            "-c" | "--cases" => cases_path = "".into(),
            unknown => panic!("unknown arg: {unknown}"),
        }
    }

    assert!(!cases_path.is_empty(), "missing path for the cases option");

    if accept_flag {
        accept(&PathBuf::from(cases_path))
    }
}

fn accept(cases_dir: &Path) {
    let test_cases = std::fs::read_dir(cases_dir).unwrap();

    for test_case in test_cases {
        let test_case = test_case.unwrap();

        if test_case.file_name().to_string_lossy().ends_with('_') {
            println!(
                "{} (ignored because case names ends with `_`)",
                test_case.path().display()
            );
            continue;
        }
        println!("{}", test_case.path().display());

        let source_paths: Vec<_> = std::fs::read_dir(test_case.path())
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

        for source_path in source_paths {
            let source = std::fs::read_to_string(&source_path).unwrap();

            let source_extension = source_path.extension().unwrap().display().to_string();
            let (transformed, diagnostics) = match &*source_extension {
                "ddsl" => {
                    let (transformed, diagnostics) =
                        device_driver_core::compile(&source, get_compile_options()).unwrap();
                    let mut diagnostics_output = String::new();

                    diagnostics
                        .print_to_fmt(
                            &mut diagnostics_output,
                            Metadata {
                                source: &source,
                                source_path: "input.ddsl",
                                term_width: None,
                                ansi: false,
                                unicode: false,
                                anonymized_line_numbers: true,
                            },
                        )
                        .unwrap();
                    (transformed, diagnostics_output)
                }
                e => panic!("Unrecognized extension: {e:?}"),
            };

            let diagnostics_path = source_path
                .with_file_name("diagnostics")
                .with_extension("txt");
            std::fs::write(
                diagnostics_path,
                device_driver_tests::normalize_test_string(&diagnostics),
            )
            .unwrap();

            let output = device_driver_tests::OUTPUT_HEADER.to_string() + &transformed;
            let output_name = format!("{}.rs", test_case.file_name().display());
            let output_path = test_case.path().join(output_name);

            std::fs::write(&output_path, output).unwrap();

            let stderr = device_driver_tests::compile_output(&output_path);
            let stderr_path = test_case.path().join("stderr.rs.txt");
            if stderr.is_empty() {
                let _ = std::fs::remove_file(stderr_path);
            } else {
                std::fs::write(
                    stderr_path,
                    device_driver_tests::normalize_test_string(&stderr),
                )
                .unwrap();
            }
        }
    }
}
