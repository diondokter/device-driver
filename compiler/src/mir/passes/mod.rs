use miette::bail;

use crate::mir::RepeatSource;

use super::{Device, Object, Repeat};

pub mod address_types_big_enough;
pub mod address_types_specified;
pub mod base_types_specified;
pub mod bit_order_specified;
pub mod bit_ranges_validated;
pub mod bool_fields_checked;
pub mod byte_order_specified;
pub mod device_name_is_pascal;
pub mod enum_values_checked;
pub mod extern_values_checked;
pub mod field_conversion_valid;
pub mod names_normalized;
pub mod names_unique;
pub mod repeat_with_enums_checked;
pub mod reset_values_converted;

pub fn run_passes(device: &mut Device) -> miette::Result<()> {
    bit_order_specified::run_pass(device)?;
    base_types_specified::run_pass(device)?;
    device_name_is_pascal::run_pass(device)?;
    names_normalized::run_pass(device)?;
    names_unique::run_pass(device)?;
    enum_values_checked::run_pass(device)?;
    repeat_with_enums_checked::run_pass(device)?;
    extern_values_checked::run_pass(device)?;
    field_conversion_valid::run_pass(device)?;
    byte_order_specified::run_pass(device)?;
    reset_values_converted::run_pass(device)?;
    bool_fields_checked::run_pass(device)?;
    bit_ranges_validated::run_pass(device)?;
    address_types_specified::run_pass(device)?;
    address_types_big_enough::run_pass(device)?;

    Ok(())
}

pub(crate) fn recurse_objects_mut(
    objects: &mut [Object],
    f: &mut impl FnMut(&mut Object) -> miette::Result<()>,
) -> miette::Result<()> {
    recurse_objects_with_depth_mut(objects, &mut |o, _| f(o))
}

pub(crate) fn recurse_objects_with_depth_mut(
    objects: &mut [Object],
    f: &mut impl FnMut(&mut Object, usize) -> miette::Result<()>,
) -> miette::Result<()> {
    fn recurse_objects_with_depth_mut(
        objects: &mut [Object],
        f: &mut impl FnMut(&mut Object, usize) -> miette::Result<()>,
        depth: usize,
    ) -> miette::Result<()> {
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
    f: &mut impl FnMut(&'o Object) -> miette::Result<()>,
) -> miette::Result<()> {
    recurse_objects_with_depth(objects, &mut |o, _| f(o))
}

pub(crate) fn recurse_objects_with_depth<'o>(
    objects: &'o [Object],
    f: &mut impl FnMut(&'o Object, usize) -> miette::Result<()>,
) -> miette::Result<()> {
    fn recurse_objects_with_depth_inner<'o>(
        objects: &'o [Object],
        f: &mut impl FnMut(&'o Object, usize) -> miette::Result<()>,
        depth: usize,
    ) -> miette::Result<()> {
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

pub(crate) fn search_object<'o>(objects: &'o [Object], name: &str) -> Option<&'o Object> {
    let mut found_object = None;

    let _ = recurse_objects(objects, &mut |object| {
        if object.name() == name {
            found_object = Some(object);
            // We want to shortcircuit for performance. The only way that can be done is by returning an error
            bail!("");
        }
        Ok(())
    });

    found_object
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
                source: RepeatSource::Count(1),
                stride: 0,
            });

            let total_address_offsets = address_offsets.iter().sum::<i128>();

            match repeat.source {
                RepeatSource::Count(count) => {
                    let count_0_address = total_address_offsets + address;
                    let count_max_address =
                        count_0_address + (count.saturating_sub(1) as i128 * repeat.stride);

                    min_address_found = min_address_found
                        .min(count_0_address)
                        .min(count_max_address);
                    max_address_found = max_address_found
                        .max(count_0_address)
                        .max(count_max_address);
                }
                RepeatSource::Enum(enum_name) => {
                    let enum_value = search_object(objects, &enum_name)
                        .expect("A mir pass checked this enum exists")
                        .as_enum()
                        .expect("A mir pass checked this is an enum");

                    for (discriminant, _) in enum_value.iter_variants_with_discriminant() {
                        let address =
                            total_address_offsets + address + (discriminant * repeat.stride);
                        min_address_found = min_address_found.min(address);
                        max_address_found = max_address_found.max(address);
                    }
                }
            }
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
