use super::recurse_objects_mut;
use crate::mir::{BaseType, Device};
use anyhow::ensure;

/// Check all bool fields. They must be exactly one bit long and have no conversion
pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    recurse_objects_mut(&mut device.objects, &mut |object| {
        for field in object.field_sets().flatten() {
            if field.base_type == BaseType::Bool {
                ensure!(
                    field.field_address.clone().count() == 1,
                    "Object \"{}\" has field \"{}\" which is of base type `bool` and is larger than 1 bit. A bool can only be one bit.",
                    object.name(),
                    field.name
                );

                ensure!(
                    field.field_conversion.is_none(),
                    "Object \"{}\" has field \"{}\" which is of base type `bool` and has specified a conversion. This is not supported for bools.",
                    object.name(),
                    field.name
                );
            }
        }

        Ok(())
    })
}
