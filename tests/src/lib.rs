use std::{
    ops::Deref,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

pub const OUTPUT_HEADER: &str = include_str!("output_header.txt");

include!(concat!(env!("OUT_DIR"), "/test_cases.rs"));

pub fn run_test(input_paths: &[&Path], output_path: &Path) {
    set_miette_hook();

    let expected_output = std::fs::read_to_string(output_path).unwrap();

    for input_path in input_paths {
        let input = std::fs::read_to_string(input_path).unwrap();

        let input_extension = input_path.extension().unwrap().display().to_string();
        let (transformed, diagnostics) = match input_extension.deref() {
            "yaml" => (
                device_driver_generation::transform_yaml(&input, "Device"),
                "".to_string(),
            ),
            "kdl" => match device_driver_generation::transform_kdl(
                &input,
                input_path
                    .strip_prefix(std::env::current_dir().unwrap())
                    .unwrap(),
            ) {
                Ok((transformed, diagnostics)) => (transformed, diagnostics.to_string()),
                Err(diagnostics) => ("".to_string(), diagnostics.to_string()),
            },
            e => panic!("Unrecognized extension: {e:?}"),
        };

        let output = OUTPUT_HEADER.to_string() + &transformed;

        let diagnostics_path = input_path
            .with_file_name("diagnostics")
            .with_extension(input_extension + ".txt");
        let expected_diagnostics = std::fs::read_to_string(&diagnostics_path).unwrap();

        pretty_assertions::assert_str_eq!(
            expected_diagnostics,
            diagnostics,
            "Diagnostics are not equal: {}",
            diagnostics_path.display()
        );

        pretty_assertions::assert_str_eq!(
            expected_output,
            output,
            "Failed output file: {}",
            output_path.display()
        );
    }

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
    cmd.env("CARGO_TARGET_DIR", "../target");

    let output = cmd.output().unwrap();

    if !output.status.success() {
        panic!(
            "Could not compile output file {}:\n{}",
            output_path.display(),
            String::from_utf8(output.stderr).unwrap()
        );
    }
}

pub fn set_miette_hook() {
    static INITIALIZED: AtomicBool = AtomicBool::new(false);

    if INITIALIZED
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_ok()
    {
        miette::set_hook(Box::new(|_| {
            Box::new(
                miette::MietteHandlerOpts::new()
                    .graphical_theme(miette::GraphicalTheme {
                        characters: {
                            let mut unicode = miette::ThemeCharacters::unicode();
                            unicode.error = "error:".into();
                            unicode.warning = "warning:".into();
                            unicode.advice = "advice:".into();
                            unicode
                        },
                        styles: miette::ThemeStyles::none(),
                    })
                    .terminal_links(false)
                    .width(120)
                    .build(),
            )
        }))
        .unwrap();
    }
}
