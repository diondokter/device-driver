use clap::Parser;
use device_driver_diagnostics::DynError;
use device_driver_lir::model::Driver;

use crate::File;

#[derive(Parser, Debug, Clone, Default)]
#[command(no_binary_name = true, bin_name = "")]
pub struct DocsCodegenOptions {}

pub fn codegen(
    codegen_options: &DocsCodegenOptions,
    lir_driver: &Driver,
    source: &str,
) -> Result<Vec<File>, DynError> {
    todo!()
}
