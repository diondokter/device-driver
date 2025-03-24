use clap::Parser;
use std::{error::Error, io::Write, ops::Deref, path::PathBuf};

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

    let output = match extension.deref() {
        "json" => device_driver_generation::transform_json(
            &manifest_contents,
            &args.device_name.to_string(),
        ),
        "yaml" => device_driver_generation::transform_yaml(
            &manifest_contents,
            &args.device_name.to_string(),
        ),
        "toml" => device_driver_generation::transform_toml(
            &manifest_contents,
            &args.device_name.to_string(),
        ),
        "dsl" => device_driver_generation::transform_dsl(
            syn::parse_str(&manifest_contents).expect("Could not (syn) parse the DSL"),
            &args.device_name.to_string(),
        ),
        unknown => panic!(
            "Unknown manifest file extension: '{unknown}'. Only 'dsl', 'json', 'yaml' and 'toml' are allowed."
        ),
    };

    let pretty_output = prettyplease::unparse(&syn::parse2(output).unwrap());

    let output: &mut dyn Write = match &args.output_path {
        Some(path) => &mut std::fs::File::create(path)
            .unwrap_or_else(|_| panic!("Could not create the output file at: {}. Does its directory exist?", path.display())),
        None => &mut std::io::stdout().lock(),
    };

    let mut output = std::io::BufWriter::new(output);
    output
        .write_all(pretty_output.as_bytes())
        .expect("Could not write the output");

    if pretty_output.starts_with("::core::compile_error!") {
        let error_output = pretty_output
            .strip_prefix("::core::compile_error!(\"")
            .unwrap()
            .strip_suffix("\");\n")
            .unwrap();
        return Err(error_output.into());
    }

    Ok(())
}
