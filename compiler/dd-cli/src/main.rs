use clap::{Parser, Subcommand};
use device_driver_core::Target;
use device_driver_diagnostics::{DynError, Metadata, ResultExt};
use std::{io::Write, path::PathBuf, process::ExitCode};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Compile DDSL to the target output
    Build(BuildArgs),
    /// Generate docs
    #[cfg(feature = "gen-docs")]
    GenDocs(GenDocsArgs),
}

#[derive(Parser, Debug)]
struct BuildArgs {
    /// Path to the input file.
    #[arg(short = 's', long = "source", value_name = "FILE", global = true)]
    source_path: Option<PathBuf>,
    /// Path to output location. Any existing file is overwritten. If not provided, the output is written to stdout.
    #[arg(short = 'o', long = "output", value_name = "FILE", global = true)]
    output_path: Option<PathBuf>,
    /// Type of generated output
    #[command(subcommand)]
    target: Target,
}

#[derive(Parser, Debug)]
struct GenDocsArgs {
    /// Path to output folder location
    #[arg(short = 'o', long = "output", value_name = "DIR")]
    output_path: PathBuf,
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

    match args.command {
        Command::Build(args) => build(args),
        #[cfg(feature = "gen-docs")]
        Command::GenDocs(args) => gen_docs(args),
    }
}

fn build(args: BuildArgs) -> Result<ExitCode, DynError> {
    let target = args.target;

    let Some(source_path) = args.source_path else {
        return Err(DynError::new("no source path provided"));
    };

    let source = std::fs::read_to_string(&source_path)
        .with_message(|| format!("Failed to open input file at: {:?}", source_path.display()))?;

    let (output, diagnostics) = device_driver_core::compile(&source, target)
        .with_message(|| "internal compilation error")?;

    let diagnostics_has_error = diagnostics.has_error();

    diagnostics
        .print_to(
            std::io::stderr().lock(),
            Metadata {
                source: &source,
                source_path: &source_path.display().to_string(),
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

#[cfg(feature = "gen-docs")]
fn gen_docs(args: GenDocsArgs) -> Result<ExitCode, DynError> {
    device_driver_core::gen_docs(&args.output_path).map(|()| ExitCode::SUCCESS)
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum TargetKind {
    Rust,
}
