use std::collections::HashSet;

use crate::{
    mir::{Device, LendingIterator, Manifest, RepeatSource, Unique, UniqueId},
    reporting::Diagnostics,
};

use super::{Object, Repeat};

pub mod address_types_big_enough;
pub mod address_types_specified;
pub mod addresses_non_overlapping;
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

pub fn run_passes(manifest: &mut Manifest, diagnostics: &mut Diagnostics) -> miette::Result<()> {
    bit_order_specified::run_pass(manifest);
    base_types_specified::run_pass(manifest, diagnostics);
    device_name_is_pascal::run_pass(manifest, diagnostics);
    names_normalized::run_pass(manifest);
    names_unique::run_pass(manifest, diagnostics);
    let removals = enum_values_checked::run_pass(manifest, diagnostics);
    remove_objects(manifest, removals);
    repeat_with_enums_checked::run_pass(manifest, diagnostics);
    let removals = extern_values_checked::run_pass(manifest, diagnostics);
    remove_objects(manifest, removals);
    let removals = field_conversion_valid::run_pass(manifest, diagnostics);
    remove_objects(manifest, removals);
    byte_order_specified::run_pass(manifest, diagnostics);
    reset_values_converted::run_pass(manifest, diagnostics);
    bool_fields_checked::run_pass(manifest, diagnostics);
    let removals = bit_ranges_validated::run_pass(manifest, diagnostics);
    remove_objects(manifest, removals);
    let removals = address_types_specified::run_pass(manifest, diagnostics);
    remove_objects(manifest, removals);
    let removals = address_types_big_enough::run_pass(manifest, diagnostics);
    remove_objects(manifest, removals);
    addresses_non_overlapping::run_pass(manifest, diagnostics);

    Ok(())
}

pub(crate) fn search_object<'o>(manifest: &'o Manifest, name: &str) -> Option<&'o Object> {
    manifest.iter_objects().find(|o| o.name() == name)
}

/// Returns None if device has no objects that pass the filter
#[expect(clippy::type_complexity, reason = "I disagree")]
pub(crate) fn find_min_max_addresses<'m>(
    manifest: &'m Manifest,
    device: &'m Device,
    filter: impl Fn(&'m Object) -> bool,
) -> Option<((i128, &'m Object), (i128, &'m Object))> {
    let mut min_address_found = i128::MAX;
    let mut min_obj_found = None;
    let mut max_address_found = i128::MIN;
    let mut max_obj_found = None;

    let mut children_left = vec![device.objects.len()];
    let mut address_offsets = vec![0];

    for object in device.iter_objects() {
        while children_left.last() == Some(&0) {
            children_left.pop();
            address_offsets.pop();
        }

        *children_left.last_mut().unwrap() -= 1;

        if !filter(object) {
            continue;
        }

        if let Some(address) = object.address() {
            let repeat = object.repeat().cloned().unwrap_or(Repeat {
                source: RepeatSource::Count(1),
                stride: 0,
            });

            let total_address_offsets = address_offsets.iter().sum::<i128>();

            match repeat.source {
                RepeatSource::Count(count) => {
                    let count_0_address = total_address_offsets + address.value;
                    let count_max_address =
                        count_0_address + (count.saturating_sub(1) as i128 * repeat.stride);
                    let min_address = count_0_address.min(count_max_address);
                    let max_address = count_0_address.max(count_max_address);

                    if min_address < min_address_found {
                        min_address_found = min_address;
                        min_obj_found = Some(object);
                    }

                    if max_address > max_address_found {
                        max_address_found = max_address;
                        max_obj_found = Some(object);
                    }
                }
                RepeatSource::Enum(enum_name) => {
                    let enum_value = search_object(manifest, &enum_name)
                        .expect("A mir pass checked this enum exists")
                        .as_enum()
                        .expect("A mir pass checked this is an enum");

                    for (discriminant, _) in enum_value.iter_variants_with_discriminant() {
                        let address =
                            total_address_offsets + address.value + (discriminant * repeat.stride);
                        if address < min_address_found {
                            min_address_found = address;
                            min_obj_found = Some(object);
                        }

                        if address > max_address_found {
                            max_address_found = address;
                            max_obj_found = Some(object);
                        }
                    }
                }
            }
        }

        match object {
            Object::Device(d) => {
                address_offsets.push(0);
                children_left.push(d.objects.len());
            }
            Object::Block(b) => {
                address_offsets.push(b.address_offset.value);
                children_left.push(b.objects.len());
            }
            _ => (),
        }
    }

    Some((
        (min_address_found, min_obj_found?),
        (max_address_found, max_obj_found?),
    ))
}

fn remove_objects(manifest: &mut Manifest, mut removals: HashSet<UniqueId>) {
    fn try_remove_from_vec(objects: &mut Vec<Object>, removals: &mut HashSet<UniqueId>) {
        removals.retain(|removal| {
            if let Some((index, _)) = objects
                .iter()
                .enumerate()
                .find(|(_, obj)| obj.has_id(removal))
            {
                objects.remove(index);
                false
            } else {
                // Find a field
                for fs in objects.iter_mut().filter_map(|o| o.as_field_set_mut()) {
                    let fs_id = fs.id();
                    for field_index in 0..fs.fields.len() {
                        if fs.fields[field_index].has_id_with(fs_id.clone(), removal) {
                            fs.fields.remove(field_index);
                            return false;
                        }
                    }
                }

                true
            }
        })
    }

    if removals.is_empty() {
        return;
    }

    try_remove_from_vec(&mut manifest.root_objects, &mut removals);

    if removals.is_empty() {
        return;
    }

    let mut iter = manifest.iter_objects_with_config_mut();
    while let Some((object, _)) = iter.next() {
        let Some(child_objects) = object.child_objects_vec() else {
            continue;
        };

        try_remove_from_vec(child_objects, &mut removals);

        if removals.is_empty() {
            return;
        }
    }
}
