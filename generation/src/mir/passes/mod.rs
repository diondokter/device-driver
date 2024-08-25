use super::{Device, Object};

mod enum_values_checked;
mod names_normalized;
mod names_unique;
mod byte_order_specified;

pub fn run_passes(device: &mut Device) -> anyhow::Result<()> {
    names_normalized::run_pass(device)?;
    names_unique::run_pass(device)?;
    enum_values_checked::run_pass(device)?;
    byte_order_specified::run_pass(device)?;

    // TODO:
    // - Reset values
    //   - Figure out the endianness for reset value arrays
    //   - Convert integer reset value to array reset value
    //   - Check if register array reset value is as long as and not longer than the size bits
    // - Validate bit ranges. Is there overlap? Is it not bigger than the register size?
    // - Validate reset value. Reject too big. Maybe already parse into byte array.
    // - Resolve and copy refs

    Ok(())
}

fn recurse_objects(
    objects: &mut [Object],
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
