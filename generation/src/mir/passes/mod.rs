use super::{Device, Object, Repeat};

mod address_types_big_enough;
mod address_types_specified;
mod base_types_specified;
mod bit_order_specified;
mod bit_ranges_validated;
mod bool_fields_checked;
mod byte_order_specified;
mod device_name_is_pascal;
mod enum_values_checked;
mod field_set_descriptions_set;
mod names_normalized;
mod names_unique;
mod propagate_cfg;
mod refs_validated;
mod reset_values_converted;

pub fn run_passes(device: &mut Device) -> anyhow::Result<()> {
    bit_order_specified::run_pass(device)?;
    device_name_is_pascal::run_pass(device)?;
    propagate_cfg::run_pass(device)?;
    names_normalized::run_pass(device)?;
    names_unique::run_pass(device)?;
    enum_values_checked::run_pass(device)?;
    byte_order_specified::run_pass(device)?;
    reset_values_converted::run_pass(device)?;
    bool_fields_checked::run_pass(device)?;
    bit_ranges_validated::run_pass(device)?;
    base_types_specified::run_pass(device)?;
    refs_validated::run_pass(device)?;
    address_types_specified::run_pass(device)?;
    address_types_big_enough::run_pass(device)?;
    field_set_descriptions_set::run_pass(device)?;

    Ok(())
}

pub(crate) fn recurse_objects_mut(
    objects: &mut [Object],
    f: &mut impl FnMut(&mut Object) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    recurse_objects_with_depth_mut(objects, &mut |o, _| f(o))
}

pub(crate) fn recurse_objects_with_depth_mut(
    objects: &mut [Object],
    f: &mut impl FnMut(&mut Object, usize) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    fn recurse_objects_with_depth_mut(
        objects: &mut [Object],
        f: &mut impl FnMut(&mut Object, usize) -> anyhow::Result<()>,
        depth: usize,
    ) -> anyhow::Result<()> {
        for object in objects.iter_mut() {
            f(object, depth)?;

            if let Some(objects) = object.get_block_object_list_mut() {
                recurse_objects_with_depth_mut(objects, f, depth + 1)?;
            }
        }

        Ok(())
    }

    recurse_objects_with_depth_mut(objects, f, 0)
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

pub(crate) fn find_min_max_addresses(
    objects: &[Object],
    filter: impl Fn(&Object) -> bool,
) -> (i128, i128) {
    let mut min_address_found = 0;
    let mut max_address_found = 0;

    let mut last_depth = 0;
    let mut address_offsets = vec![0];

    recurse_objects_with_depth(objects, &mut |object, depth| {
        while depth < last_depth {
            address_offsets.pop();
            last_depth -= 1;
        }

        if !filter(object) {
            return Ok(());
        }

        if let Some(address) = object.address() {
            let repeat = object.repeat().unwrap_or(Repeat {
                count: 1,
                stride: 0,
            });

            let total_address_offsets = address_offsets.iter().sum::<i128>();

            let count_0_address = total_address_offsets + address;
            let count_max_address =
                count_0_address + (repeat.count.saturating_sub(1) as i128 * repeat.stride);

            min_address_found = min_address_found
                .min(count_0_address)
                .min(count_max_address);
            max_address_found = max_address_found
                .max(count_0_address)
                .max(count_max_address);
        }

        if let Object::Block(b) = object {
            // Push an offset because the next objects are gonna be deeper
            address_offsets.push(b.address_offset);
            last_depth += 1;
        }

        Ok(())
    })
    .unwrap();

    (min_address_found, max_address_found)
}

pub(crate) fn search_object<'d>(name: &str, objects: &'d [Object]) -> Option<&'d Object> {
    for object in objects {
        if object.name() == name {
            return Some(object);
        }

        if let Some(block_objects) = object.get_block_object_list() {
            match search_object(name, block_objects) {
                None => {}
                found => return found,
            }
        }
    }

    None
}
