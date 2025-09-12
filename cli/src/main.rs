use clap::Parser;
use device_driver_compiler::reporting::Diagnostics;
use miette::{Context, IntoDiagnostic};
use std::{
    io::Write,
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the input file.
    input_path: PathBuf,
    /// Path to output location. Any existing file is overwritten. If not provided, the output is written to stdout.
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    output_path: Option<PathBuf>,
    /// Type of generated output
    #[arg(short = 't', long = "target", default_value = "rust")]
    target: Target,
}

fn main() -> ExitCode {
    match run() {
        Ok(exit) => exit,
        Err(error) => {
            eprintln!("{error:?}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> miette::Result<ExitCode> {
    let args = Args::parse();

    device_driver_compiler::reporting::set_miette_hook(true);

    let input_contents = std::fs::read_to_string(&args.input_path)
        .into_diagnostic()
        .wrap_err_with(|| {
            format!(
                "Trying to open input file at: {:?}",
                args.input_path.display()
            )
        })?;

    let (output, reports) = args.target.generate(&input_contents, &args.input_path);

    reports
        .print_to(std::io::stderr().lock())
        .into_diagnostic()?;

    if reports.has_error() {
        return Ok(ExitCode::FAILURE);
    }

    let output_writer: &mut dyn Write = match &args.output_path {
        Some(path) => &mut std::fs::File::create(path)
            .into_diagnostic()
            .wrap_err_with(|| {
                format!(
                    "Could not create the output file at: {:?}. Does its directory exist?",
                    path.display()
                )
            })?,
        None => &mut std::io::stdout().lock(),
    };

    let mut output_writer = std::io::BufWriter::new(output_writer);
    output_writer
        .write_all(output.as_bytes())
        .into_diagnostic()
        .wrap_err_with(|| {
            format!(
                "Could not write output to {}",
                args.output_path
                    .map(|path| format!("{:?}", path.display()))
                    .unwrap_or_else(|| "stdout".into())
            )
        })?;

    Ok(ExitCode::SUCCESS)
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum Target {
    Rust,
}

impl Target {
    pub fn generate(&self, input_contents: &str, input_path: &Path) -> (String, Diagnostics) {
        match self {
            Target::Rust => device_driver_compiler::transform_kdl(input_contents, None, input_path),
        }
    }
}
