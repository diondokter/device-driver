use clap::Parser;
use device_driver_core::Target;
use device_driver_diagnostics::{DynError, Metadata, ResultExt};
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
    target: TargetArg,
    /// Compiler options
    #[arg(
        short = 'C',
        value_name = "KEY=VALUE", 
        value_parser = parse_key_val,
        action = clap::ArgAction::Append,
    )]
    c_opts: Option<Vec<(String, String)>>,
}

/// Parse a single key-value pair
fn parse_key_val(s: &str) -> Result<(String, String), &'static str> {
    s.split_once('=')
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .ok_or("no `=` found`")
}

fn main() -> ExitCode {
    match run() {
        Ok(exit) => exit,
        Err(error) => {
            eprintln!("{error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<ExitCode, DynError> {
    let args = Args::parse();

    let target: Target = args.target.into();
    let mut compile_options = target.get_compile_options();

    for (key, value) in args.c_opts.unwrap_or_default() {
        if !compile_options.add(&key, value) {
            return Err(DynError::new(format!("Unknown compiler flag: `{key}`")));
        }
    }

    let source = std::fs::read_to_string(&args.source_path).with_message(|| {
        format!(
            "Failed to open input file at: {:?}",
            args.source_path.display()
        )
    })?;

    let (output, diagnostics) = device_driver_core::compile(&source, target, compile_options)
        .with_message(|| "internal compilation error")?;

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
        .into_dyn_result()?;

    if diagnostics_has_error {
        return Ok(ExitCode::FAILURE);
    }

    let output_writer: &mut dyn Write = match &args.output_path {
        Some(path) => &mut std::fs::File::create(path).with_message(|| {
            format!(
                "could not create the output file at: {:?}. Does its directory exist?",
                path.display()
            )
        })?,
        None => &mut std::io::stdout().lock(),
    };

    let mut output_writer = std::io::BufWriter::new(output_writer);
    output_writer
        .write_all(output.as_bytes())
        .with_message(|| {
            format!(
                "could not write output to {}",
                args.output_path
                    .map_or_else(|| "stdout".into(), |path| format!("{:?}", path.display()))
            )
        })?;

    Ok(ExitCode::SUCCESS)
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum TargetArg {
    Rust,
}

impl From<TargetArg> for Target {
    fn from(value: TargetArg) -> Self {
        match value {
            TargetArg::Rust => Self::Rust,
        }
    }
}
