use super::recurse_objects_mut;
use crate::mir::{BaseType, Device};
use anyhow::ensure;

/// Check all bool fields. They must be exactly zero or one bits long and have no conversion
pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    recurse_objects_mut(&mut device.objects, &mut |object| {
        let object_name = object.name().to_string();

        for field in object.field_sets_mut().flatten() {
            if field.base_type == BaseType::Bool {
                // When zero bits long, extend to one bit
                if field.field_address.start == field.field_address.end {
                    field.field_address.end += 1;
                }

                ensure!(
                    field.field_address.clone().count() == 1,
                    "Object \"{}\" has field \"{}\" which is of base type `bool` and is larger than 1 bit. A bool can only be zero or one bit.",
                    object_name,
                    field.name
                );

                ensure!(
                    field.field_conversion.is_none(),
                    "Object \"{}\" has field \"{}\" which is of base type `bool` and has specified a conversion. This is not supported for bools.",
                    object_name,
                    field.name
                );
            }
        }

        Ok(())
    })
}
