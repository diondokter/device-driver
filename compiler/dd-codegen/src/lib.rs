use clap::Subcommand;
use device_driver_diagnostics::DynError;
use device_driver_lir::model::Driver;
use itertools::Itertools;

pub use crate::docs::DocsCodegenOptions;
pub use crate::rust::RustCodegenOptions;

mod docs;
mod rust;

#[derive(Debug, Clone, Subcommand)]
pub enum Target {
    /// Generate Rust code
    Rust(RustCodegenOptions),
    /// Generate a documentation website
    Docs(DocsCodegenOptions),
}

impl Target {
    pub fn create_error_message(&self) -> &'static str {
        match self {
            Target::Rust(_) => {
                "compile_error!(\"The device driver input has errors that need to be solved!\");"
            }
            Target::Docs(_) => {
                "<label>The device driver input has errors that need to be solved!<label>"
            }
        }
    }

    /// Converts the multiline text to comments that work for the target
    pub fn to_comments(&self, text: &str) -> String {
        match self {
            Target::Rust(_) => text.lines().map(|line| format!("// {line}")).join("\n"),
            Target::Docs(_) => format!("<!--\n{text}\n-->"),
        }
    }
}

pub fn codegen(target: &Target, lir_driver: &Driver, source: &str) -> Result<Vec<File>, DynError> {
    match target {
        Target::Rust(codegen_options) => rust::codegen(codegen_options, lir_driver, source),
        Target::Docs(codegen_options) => docs::codegen(codegen_options, lir_driver, source),
    }
}

pub struct File {
    /// The name + extension of the generated file
    pub name: String,
    /// The content of the file
    pub contents: String,
}
