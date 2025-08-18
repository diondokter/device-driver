use crate::mir::Device;

use super::recurse_objects_mut;

/// For any (inline) fieldset that doesn't have a description, use the description of the parent object
pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    recurse_objects_mut(&mut device.objects, &mut |object| {
        if !object.description().is_empty() {
            let description = object.description().to_owned();

            object.field_sets_mut().for_each(|fs| {
                if fs.description.is_empty() {
                    fs.description = description.clone();
                }
            });
        }
        Ok(())
    })
}
