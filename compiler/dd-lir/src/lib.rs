mod lowering;
pub mod model;

pub fn lower_mir(manifest: device_driver_mir::model::Manifest) -> model::Driver {
    let enums = lowering::transform_enums(&manifest);
    let field_sets = lowering::transform_field_sets(&manifest);
    let devices = lowering::transform_devices(&manifest);

    model::Driver {
        devices,
        field_sets,
        enums,
    }
}
