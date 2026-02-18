use clap::Parser;
use device_driver_diagnostics::{Diagnostics, Metadata};
use miette::{Context, IntoDiagnostic};
use std::{io::Write, path::PathBuf, process::ExitCode};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the input file.
    source_path: PathBuf,
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

    device_driver_diagnostics::set_miette_hook(true);

    let source = std::fs::read_to_string(&args.source_path)
        .into_diagnostic()
        .wrap_err_with(|| {
            format!(
                "Trying to open input file at: {:?}",
                args.source_path.display()
            )
        })?;

    let (output, diagnostics) = args.target.generate(&source);

    let diagnostics_has_error = diagnostics.has_error();

    diagnostics
        .print_to(
            std::io::stderr().lock(),
            Metadata {
                source: &source,
                source_path: &args.source_path.display().to_string(),
                term_width: None,
                ansi: true,
                unicode: true,
                anonymized_line_numbers: false,
            },
        )
        .into_diagnostic()?;

    if diagnostics_has_error {
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
                    .map_or_else(|| "stdout".into(), |path| format!("{:?}", path.display()))
            )
        })?;

    Ok(ExitCode::SUCCESS)
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum Target {
    Rust,
}

impl Target {
    #[must_use]
    pub fn generate(&self, source: &str) -> (String, Diagnostics) {
        match self {
            Target::Rust => device_driver_core::compile(source, None),
        }
    }
}
