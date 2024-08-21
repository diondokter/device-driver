use super::{Device, Object};

mod enum_values_specified;
mod names_normalized;
mod names_unique;

pub fn run_passes(device: &mut Device) -> anyhow::Result<()> {
    names_normalized::run_pass(device)?;
    names_unique::run_pass(device)?;
    // enum_values_specified::run_pass(device)?;

    Ok(())
}

pub(self) fn recurse_objects(
    objects: &mut Vec<Object>,
    f: &mut impl FnMut(&mut Object) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    for object in objects.iter_mut() {
        f(object)?;

        if let Some(objects) = object.get_block_object_list() {
            recurse_objects(objects, f)?;
        }
    }

    Ok(())
}
