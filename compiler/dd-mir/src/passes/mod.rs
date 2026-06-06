use std::{any::type_name, collections::HashSet};

use crate::{
    model::{LendingIterator, Manifest, Object, Unique, UniqueId},
    passes::{
        address_types_big_enough::AddressTypesBigEnough,
        address_types_specified::AddressTypesSpecified,
        addresses_non_overlapping::AddressesNonOverlapping,
        base_types_specified::BaseTypesSpecified, bit_ranges_validated::BitRangesValidated,
        bool_fields_checked::BoolFieldsChecked, byte_order_specified::ByteOrderSpecified,
        device_configs_owned::DeviceConfigsOwned, device_name_is_pascal::DeviceNameIsPascal,
        enum_values_checked::EnumValuesChecked, extern_values_checked::ExternValuesChecked,
        field_conversion_valid::FieldConversionValid, field_set_refs_valid::FieldsetRefsValid,
        names_checked::NamesChecked, names_unique::NamesUnique,
        repeat_with_enums_checked::RepeatWithEnumsChecked,
        repeat_zero_stride_rejected::RepeatZeroStrideRejected,
        reset_values_converted::ResetValuesConverted,
    },
};
use device_driver_diagnostics::{Diagnostics, DynError, ResultExt};

mod address_types_big_enough;
mod address_types_specified;
mod addresses_non_overlapping;
mod base_types_specified;
mod bit_ranges_validated;
mod bool_fields_checked;
mod byte_order_specified;
mod device_configs_owned;
mod device_name_is_pascal;
mod enum_values_checked;
mod extern_values_checked;
mod field_conversion_valid;
mod field_set_refs_valid;
mod names_checked;
mod names_unique;
mod repeat_with_enums_checked;
mod repeat_zero_stride_rejected;
mod reset_values_converted;

pub fn run_passes(manifest: &mut Manifest, diagnostics: &mut Diagnostics) -> Result<(), DynError> {
    run_pass::<DeviceConfigsOwned>(manifest, diagnostics)?;
    run_pass::<BaseTypesSpecified>(manifest, diagnostics)?;
    run_pass::<DeviceNameIsPascal>(manifest, diagnostics)?;
    run_pass::<NamesChecked>(manifest, diagnostics)?;
    run_pass::<NamesUnique>(manifest, diagnostics)?;
    run_pass::<FieldsetRefsValid>(manifest, diagnostics)?;
    run_pass::<EnumValuesChecked>(manifest, diagnostics)?;
    run_pass::<RepeatZeroStrideRejected>(manifest, diagnostics)?;
    run_pass::<RepeatWithEnumsChecked>(manifest, diagnostics)?;
    run_pass::<ExternValuesChecked>(manifest, diagnostics)?;
    run_pass::<FieldConversionValid>(manifest, diagnostics)?;
    run_pass::<ByteOrderSpecified>(manifest, diagnostics)?;
    run_pass::<ResetValuesConverted>(manifest, diagnostics)?;
    run_pass::<BoolFieldsChecked>(manifest, diagnostics)?;
    run_pass::<BitRangesValidated>(manifest, diagnostics)?;
    run_pass::<AddressTypesSpecified>(manifest, diagnostics)?;
    run_pass::<AddressTypesBigEnough>(manifest, diagnostics)?;
    run_pass::<AddressesNonOverlapping>(manifest, diagnostics)?;

    Ok(())
}

fn run_pass<P: Pass>(
    manifest: &mut Manifest,
    diagnostics: &mut Diagnostics,
) -> Result<(), DynError> {
    let removals = P::run_pass(manifest, diagnostics)
        .with_message(|| format!("could not finish {} MIR pass", type_name::<P>()))?;
    remove_objects(manifest, removals);
    Ok(())
}

trait Pass {
    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<UniqueId>, DynError>;
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
