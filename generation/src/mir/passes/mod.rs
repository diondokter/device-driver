use super::{Device, Object};

mod bit_ranges_validated;
mod bool_fields_checked;
mod byte_order_specified;
mod enum_values_checked;
mod names_normalized;
mod names_unique;
mod refs_validated;
mod reset_values_converted;

pub fn run_passes(device: &mut Device) -> anyhow::Result<()> {
    names_normalized::run_pass(device)?;
    names_unique::run_pass(device)?;
    enum_values_checked::run_pass(device)?;
    byte_order_specified::run_pass(device)?;
    reset_values_converted::run_pass(device)?;
    bit_ranges_validated::run_pass(device)?;
    refs_validated::run_pass(device)?;
    bool_fields_checked::run_pass(device)?;

    // TODO:
    // - Validate address overlap. But likely only the actual address and not partial overlap
    // - Resolve refs
    // - Check if address types arae specified

    Ok(())
}

pub(crate) fn recurse_objects_mut(
    objects: &mut [Object],
    f: &mut impl FnMut(&mut Object) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    for object in objects.iter_mut() {
        f(object)?;

        if let Some(objects) = object.get_block_object_list_mut() {
            recurse_objects_mut(objects, f)?;
        }
    }

    Ok(())
}

pub(crate) fn recurse_objects(
    objects: &[Object],
    f: &mut impl FnMut(&Object) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    for object in objects.iter() {
        f(object)?;

        if let Some(objects) = object.get_block_object_list() {
            recurse_objects(objects, f)?;
        }
    }

    Ok(())
}
