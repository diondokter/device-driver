use std::{
    ops::Deref,
    path::Path,
    sync::{
        LazyLock,
        atomic::{AtomicBool, Ordering},
    },
};

use regex::Regex;

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
            "kdl" => {
                let (transformed, diagnostics) = device_driver_generation::transform_kdl(
                    &input,
                    input_path
                        .strip_prefix(std::env::current_dir().unwrap())
                        .unwrap(),
                );
                (transformed, diagnostics.to_string())
            }
            e => panic!("Unrecognized extension: {e:?}"),
        };

        let output = OUTPUT_HEADER.to_string() + &transformed;

        let diagnostics_path = input_path
            .with_file_name("diagnostics")
            .with_extension(input_extension + ".txt");
        let expected_diagnostics = std::fs::read_to_string(&diagnostics_path).unwrap();

        println!("Expected: {}", expected_diagnostics.contains("\r\n"));
        println!("generated: {}", diagnostics.contains("\r\n"));

        pretty_assertions::assert_str_eq!(
            normalize_test_string(&expected_diagnostics),
            normalize_test_string(&diagnostics),
            "Diagnostics are not equal: {}",
            diagnostics_path.display()
        );

        pretty_assertions::assert_str_eq!(
            normalize_test_string(&expected_output),
            normalize_test_string(&output),
            "Failed output file: {}",
            output_path.display()
        );
    }

    let output_extension = output_path.extension().unwrap().to_str().unwrap();
    match output_extension {
        "rs" => {
            let stderr = compile_output(output_path);
            let expected_stderr =
                std::fs::read_to_string(output_path.with_file_name("stderr.rs.txt"))
                    .unwrap_or_default();

            pretty_assertions::assert_str_eq!(expected_stderr, stderr, "Different stderr");
        }
        _ => unimplemented!(),
    }
}

pub fn compile_output(output_path: &Path) -> String {
    let mut cmd = std::process::Command::new("cargo");

    cmd.arg("+nightly");
    cmd.arg("-Zscript");
    cmd.arg(output_path);
    cmd.env("CARGO_TARGET_DIR", "../target");

    let output = cmd.output().unwrap();

    String::from_utf8_lossy(&output.stderr).to_string()
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

static DIAGNOSTICS_PATH_SEPARATOR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("\\[cases.*:.*:.*\\]").unwrap());

pub fn normalize_test_string(val: &str) -> String {
    let val = normalize_paths(val);
    normalize_line_endings(&val)
}

fn normalize_paths(val: &str) -> String {
    DIAGNOSTICS_PATH_SEPARATOR_REGEX
        .replace_all(val, |caps: &regex::Captures| caps[0].replace("\\", "/"))
        .to_string()
}

fn normalize_line_endings(val: &str) -> String {
    val.replace("\r\n", "\n")
}
