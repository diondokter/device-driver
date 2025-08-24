use clap::Parser;
use device_driver_generation::reporting::Diagnostics;
use std::{
    error::Error,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the manifest file.
    /// The type of file is determined from the file extension.
    #[arg(short = 'm', long = "manifest", value_name = "FILE")]
    manifest_path: PathBuf,
    /// Path to output location. Any existing file is overwritten. If not provided, the output is written to stdout.
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    output_path: Option<PathBuf>,
    /// Type of generated output
    #[arg(short = 'g', long = "gen-type", default_value = "rust")]
    gen_type: GenType,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

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
                    styles: miette::ThemeStyles::rgb(),
                })
                .build(),
        )
    }))
    .unwrap();

    let extension = args
        .manifest_path
        .extension()
        .map(|ext| ext.to_string_lossy())
        .expect("Manifest file has no file extension");

    let manifest_contents = std::fs::read_to_string(&args.manifest_path).unwrap_or_else(|_| {
        panic!(
            "Trying to open manifest file at: \"{}\"",
            args.manifest_path.display()
        )
    });

    let (output, reports) =
        match args
            .gen_type
            .generate(&manifest_contents, &extension, &args.manifest_path)
        {
            Ok((output, reports)) => (Some(output), reports),
            Err(reports) => (None, reports),
        };

    reports.print_to(std::io::stderr().lock()).unwrap();

    if let Some(output) = output {
        let output_writer: &mut dyn Write = match &args.output_path {
            Some(path) => &mut std::fs::File::create(path).unwrap_or_else(|_| {
                panic!(
                    "Could not create the output file at: {}. Does its directory exist?",
                    path.display()
                )
            }),
            None => &mut std::io::stdout().lock(),
        };

        let mut output_writer = std::io::BufWriter::new(output_writer);
        output_writer
            .write_all(output.as_bytes())
            .expect("Could not write the output");

        if output.starts_with("::core::compile_error!") {
            return Err(strip_compile_error(&output).into());
        }

        Ok(())
    } else {
        Err("Compilation ended with errors".to_string().into())
    }
}

fn strip_compile_error(mut error: &str) -> &str {
    error = error
        .strip_prefix("::core::compile_error!(")
        .unwrap_or(error)
        .trim();
    error = error.strip_prefix("\"").unwrap_or(error).trim();
    error = error.strip_suffix(";").unwrap_or(error).trim();
    error = error.strip_suffix(")").unwrap_or(error).trim();
    error = error.strip_suffix(",").unwrap_or(error).trim();
    error = error.strip_suffix("\"").unwrap_or(error).trim();

    error
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum GenType {
    Rust,
}

impl GenType {
    pub fn generate(
        &self,
        manifest_contents: &str,
        manifest_type: &str,
        manifest_path: &Path,
    ) -> Result<(String, Diagnostics), Diagnostics> {
        match self {
            GenType::Rust => match manifest_type {
                "kdl" => Ok(device_driver_generation::transform_kdl(
                    manifest_contents,
                    manifest_path,
                )),
                unknown => {
                    panic!("Unknown manifest file extension: '{unknown}'. Only 'kdl' is allowed.")
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_error_stripping() {
        assert_eq!(
            strip_compile_error(
                "::core::compile_error!(\"simple key expect ':' at byte 354 line 21 column 11\");"
            ),
            "simple key expect ':' at byte 354 line 21 column 11",
        );

        assert_eq!(
            strip_compile_error(
                "::core::compile_error!(\n    \"Parsing object `DOWNLOAD_POSTMORTEM`: Parsing error for 'fields_in': Parsing field 'LENGTH': Unexpected key: 'default'\",\n)\n"
            ),
            "Parsing object `DOWNLOAD_POSTMORTEM`: Parsing error for 'fields_in': Parsing field 'LENGTH': Unexpected key: 'default'"
        );
    }
}
