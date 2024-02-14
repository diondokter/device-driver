use std::{error::Error, fs::File, io::Write, path::PathBuf};

use clap::Parser;
use device_driver_generation::Device;
use quote::quote;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The device description file
    #[arg(short, long, value_name = "FILE")]
    target_file: PathBuf,
    /// The path where the generated rust code is placed.
    /// If not specified, the current directory is used.
    #[arg(short, long, value_name = "DIR")]
    output_dir: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let extension_error_message = format!("The target file path \"{}\" has to end in `.json` or `.jsn` for json deserialization or `.yaml` or `.yml` for yaml deserialization", cli.target_file.to_string_lossy());

    let target_file = File::open(&cli.target_file)?;

    let target: Device = match cli.target_file.extension() {
        Some(extension) => match extension.to_str() {
            Some("json") | Some("jsn") => serde_json::from_reader(target_file)?,
            Some("yaml") | Some("yml") => serde_yaml::from_reader(target_file)?,
            Some(_) => return Err(extension_error_message.into()),
            None => panic!("Non-utf8 file extension"),
        },
        None => {
            return Err(extension_error_message.into());
        }
    };

    let register_functions = target.generate_device_register_functions();
    let register_functions = quote!(#(#register_functions)*).to_string();
    let register_functions = prettyplease::unparse(&syn::parse_file(&register_functions)?);

    let type_definitions = target.generate_definitions();
    let type_definitions = prettyplease::unparse(&syn::parse_file(&type_definitions.to_string())?);

    let output_dir = cli.output_dir.unwrap_or_else(|| PathBuf::from("."));

    File::create(output_dir.join("device_register_functions.rs"))?.write_all(register_functions.as_bytes())?;
    File::create(output_dir.join("device_type_definitions.rs"))?.write_all(type_definitions.as_bytes())?;

    Ok(())
}
