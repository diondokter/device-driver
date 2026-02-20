use std::collections::HashSet;

use crate::model::{LendingIterator, Manifest, Object, Unique, UniqueId};
use device_driver_diagnostics::Diagnostics;

pub mod address_types_big_enough;
pub mod address_types_specified;
pub mod addresses_non_overlapping;
pub mod base_types_specified;
pub mod bit_ranges_validated;
pub mod bool_fields_checked;
pub mod byte_order_specified;
pub mod device_name_is_pascal;
pub mod enum_values_checked;
pub mod extern_values_checked;
pub mod field_conversion_valid;
pub mod names_checked;
pub mod names_unique;
pub mod repeat_with_enums_checked;
pub mod reset_values_converted;

pub fn run_passes(manifest: &mut Manifest, diagnostics: &mut Diagnostics) {
    base_types_specified::run_pass(manifest, diagnostics);
    let removals = device_name_is_pascal::run_pass(manifest, diagnostics);
    remove_objects(manifest, removals);
    let removals = names_checked::run_pass(manifest, diagnostics);
    remove_objects(manifest, removals);
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
        });
    }

    if removals.is_empty() {
        return;
    }

    try_remove_from_vec(&mut manifest.objects, &mut removals);

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
