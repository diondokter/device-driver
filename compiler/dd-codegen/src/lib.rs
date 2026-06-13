use clap::Subcommand;
use device_driver_lir::model::Driver;

pub use crate::rust::RustCodegenOptions;

mod rust;

#[derive(Debug, Clone, Subcommand)]
pub enum Target {
    /// Generate Rust code
    Rust(RustCodegenOptions),
}

impl Target {
    pub fn create_error_message(&self) -> &'static str {
        match self {
            Target::Rust(_) => {
                "compile_error!(\"The device driver input has errors that need to be solved!\");"
            }
        }
    }
}

pub fn codegen(target: &Target, lir_driver: &Driver, source: &str) -> String {
    match target {
        Target::Rust(codegen_options) => {
            rust::DriverTemplateRust::new(lir_driver, source, codegen_options).to_string()
        }
    }
}
