use super::{Device, Object};

mod address_types_specified;
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
    address_types_specified::run_pass(device)?;

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

pub(crate) fn recurse_objects<'o>(
    objects: &'o [Object],
    f: &mut impl FnMut(&'o Object) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    recurse_objects_with_depth(objects, &mut |o, _| f(o))
}

pub(crate) fn recurse_objects_with_depth<'o>(
    objects: &'o [Object],
    f: &mut impl FnMut(&'o Object, usize) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    fn recurse_objects_with_depth_inner<'o>(
        objects: &'o [Object],
        f: &mut impl FnMut(&'o Object, usize) -> anyhow::Result<()>,
        depth: usize,
    ) -> anyhow::Result<()> {
        for object in objects.iter() {
            f(object, depth)?;

            if let Some(objects) = object.get_block_object_list() {
                recurse_objects_with_depth_inner(objects, f, depth + 1)?;
            }
        }

        Ok(())
    }

    recurse_objects_with_depth_inner(objects, f, 0)
}
