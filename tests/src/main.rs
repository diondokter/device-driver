use clap::Command;

fn main() {
    let matches = Command::new("Tests")
        .subcommand(Command::new("accept").about("Accept all changes and update the output files"))
        .get_matches();

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

        let input = std::fs::read_to_string(test_case.path().join("input.yaml")).unwrap();
        let output = tests::OUTPUT_HEADER.to_string()
            + &device_driver_generation::transform_yaml(&input, "Device");

        let output_name = format!("{}.rs", test_case.file_name().display());

        std::fs::write(test_case.path().join(output_name), output).unwrap();
    }
}
