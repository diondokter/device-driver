use device_driver_diagnostics::{DynError, ResultExt};

mod lowering;
pub mod model;

pub fn lower_mir(manifest: device_driver_mir::model::Manifest) -> Result<model::Driver, DynError> {
    let enums = lowering::transform_enums(&manifest);
    let field_sets = lowering::transform_field_sets(&manifest);
    let devices =
        lowering::transform_devices(&manifest).with_message(|| "could not transform devices")?;

    Ok(model::Driver {
        devices,
        field_sets,
        enums,
    })
}
