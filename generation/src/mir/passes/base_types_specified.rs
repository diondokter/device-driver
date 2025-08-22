use super::recurse_objects_mut;
use crate::mir::Device;

/// Turn all unspecified base types into either bools or uints based on the size of the field
pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    recurse_objects_mut(&mut device.objects, &mut |object| {
        if let Some(field_set) = object.as_field_set_mut() {
            for field in field_set.fields.iter_mut() {
                if field.base_type.is_unspecified() {
                    field.base_type = match field.field_address.len() {
                        0 => unreachable!(),
                        1 => crate::mir::BaseType::Bool,
                        _ => crate::mir::BaseType::Uint,
                    }
                }
            }
        }

        Ok(())
    })
}
