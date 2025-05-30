use clap::Parser;
use std::{error::Error, io::Write, path::PathBuf};

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
    /// The name of the device to be generated. Must be PascalCase
    #[arg(short = 'd', long = "device-name", value_name = "NAME")]
    device_name: String,
    /// Type of generated output
    #[arg(short = 'g', long = "gen-type")]
    gen_type: GenType,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

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

    let output = args
        .gen_type
        .generate(&manifest_contents, &extension, &args.device_name);

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
    Kdl,
}

impl GenType {
    pub fn generate(
        &self,
        manifest_contents: &str,
        manifest_type: &str,
        device_name: &str,
    ) -> String {
        match self {
            GenType::Rust => {
                let output = match manifest_type {
                    "json" => {
                        device_driver_generation::transform_json(&manifest_contents, device_name)
                    }
                    "yaml" => {
                        device_driver_generation::transform_yaml(&manifest_contents, device_name)
                    }
                    "toml" => {
                        device_driver_generation::transform_toml(&manifest_contents, device_name)
                    }
                    "dsl" => device_driver_generation::transform_dsl(
                        syn::parse_str(&manifest_contents).expect("Could not (syn) parse the DSL"),
                        device_name,
                    ),
                    unknown => panic!(
                        "Unknown manifest file extension: '{unknown}'. Only 'dsl', 'json', 'yaml' and 'toml' are allowed."
                    ),
                };

                prettyplease::unparse(&syn::parse_str(&output).unwrap())
            }
            GenType::Kdl => {
                let mir_device = match manifest_type {
                    "json" => {
                        device_driver_generation::_private_transform_json_mir(&manifest_contents)
                    }
                    "yaml" => {
                        device_driver_generation::_private_transform_yaml_mir(&manifest_contents)
                    }
                    "toml" => {
                        device_driver_generation::_private_transform_toml_mir(&manifest_contents)
                    }
                    "dsl" => device_driver_generation::_private_transform_dsl_mir(
                        syn::parse_str(&manifest_contents).expect("Could not (syn) parse the DSL"),
                    )
                    .map_err(|e| anyhow::Error::new(e)),
                    unknown => panic!(
                        "Unknown manifest file extension: '{unknown}'. Only 'dsl', 'json', 'yaml' and 'toml' are allowed."
                    ),
                };

                match mir_device {
                    Ok(mut mir_device) => {
                        mir_device.name = Some(device_name.to_string());
                        device_driver_generation::mir::kdl_transform::transform(mir_device)
                    }
                    Err(e) => e.to_string(),
                }
            }
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
