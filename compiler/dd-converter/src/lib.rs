use device_driver_diagnostics::DynError;
#[cfg(feature = "dd-v1")]
use device_driver_diagnostics::ResultExt;

#[cfg(feature = "dd-v1")]
mod dd_v1;

/// The format to convert into DDSL
#[derive(Debug, Clone, Copy, clap::Subcommand)]
pub enum SourceFormat {
    #[cfg(feature = "dd-v1")]
    /// The v1 formats of device-driver (DSL, YAML, JSON & TOML)
    DeviceDriverV1 {
        #[arg(long)]
        sub_format: DeviceDriverV1Format,
    },
}

#[cfg(feature = "dd-v1")]
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum DeviceDriverV1Format {
    DSL,
    YAML,
    JSON,
    TOML,
}

/// Convert the source to ddsl.
///
/// Don't use in long running programs since conversion may leak memory.
/// That's because we're running the ddsl parser in reverse while the parser is
/// optimized for speed and memory use and so usually borrows from the DDSL source.
pub fn convert(_source: &str, format: SourceFormat) -> Result<String, DynError> {
    match format {
        #[cfg(feature = "dd-v1")]
        SourceFormat::DeviceDriverV1 { sub_format } => dd_v1::convert(_source, sub_format)
            .with_message(|| format!("converting device-driver v1 format {sub_format:?}")),
    }
}
