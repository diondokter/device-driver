use clap::Command;

fn main() {
    let matches = Command::new("Tests")
        .subcommand(Command::new("accept").about("Accept all changes and update the output files"))
        .get_matches();

    device_driver_core::reporting::set_miette_hook(false);

    match matches.subcommand_name() {
        Some("accept") => accept(),
        None => println!("Choose an action to do..."),
        _ => unreachable!(),
    }
}

fn accept() {
    let test_cases = std::fs::read_dir("cases").unwrap();

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

        let input_paths: Vec<_> = std::fs::read_dir(test_case.path())
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

        for input_path in input_paths {
            let input = std::fs::read_to_string(&input_path).unwrap();

            let input_extension = input_path.extension().unwrap().display().to_string();
            let (transformed, diagnostics) = match &*input_extension {
                "kdl" => {
                    let (transformed, diagnostics) =
                        device_driver_core::transform_kdl(&input, None, &input_path);
                    (transformed, diagnostics.to_string())
                }
                e => panic!("Unrecognized extension: {e:?}"),
            };

            let diagnostics_path = input_path
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
