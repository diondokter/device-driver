use std::{any::type_name, collections::HashSet, error::Error, fmt::Display};

use crate::{
    model::{Manifest, UniqueId},
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

// TODO: Make const when possible in a future Rust version
fn get_default_passes() -> [PassInfo; 18] {
    [
        PassInfo::get::<DeviceConfigsOwned>(),
        PassInfo::get::<BaseTypesSpecified>(),
        PassInfo::get::<DeviceNameIsPascal>(),
        PassInfo::get::<NamesChecked>(),
        PassInfo::get::<NamesUnique>(),
        PassInfo::get::<FieldsetRefsValid>(),
        PassInfo::get::<EnumValuesChecked>(),
        PassInfo::get::<RepeatZeroStrideRejected>(),
        PassInfo::get::<RepeatWithEnumsChecked>(),
        PassInfo::get::<ExternValuesChecked>(),
        PassInfo::get::<FieldConversionValid>(),
        PassInfo::get::<ByteOrderSpecified>(),
        PassInfo::get::<ResetValuesConverted>(),
        PassInfo::get::<BoolFieldsChecked>(),
        PassInfo::get::<BitRangesValidated>(),
        PassInfo::get::<AddressTypesSpecified>(),
        PassInfo::get::<AddressTypesBigEnough>(),
        PassInfo::get::<AddressesNonOverlapping>(),
    ]
}

pub fn run_passes(manifest: &mut Manifest, diagnostics: &mut Diagnostics) -> Result<(), DynError> {
    // TODO: Add assumption tracking
    // TODO: Add optional randomization using the assumptions
    // - randomize function exists, just need the flag plumbing
    // - flag should be `-Z randomize-mir-passes`
    // - this needs a more generalized flag system (currently only codegen has flags)
    let passes = get_default_passes();

    check_assumptions(&passes).with_message(|| "checking mir pass assumptions")?;

    // Run the passes
    for pass in passes {
        (pass.pass)(manifest, diagnostics)?;
    }

    Ok(())
}

trait Pass {
    const ASSUMPTIONS_MADE: &[Assumption];
    const ASSUMPTIONS_RELEASED: &[Assumption];

    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<UniqueId>, DynError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Assumption {
    DeviceConfigsOwned,
}

#[derive(Debug, Clone)]
struct PassInfo {
    assumptions_made: &'static [Assumption],
    assumptions_released: &'static [Assumption],
    pass: fn(manifest: &mut Manifest, diagnostics: &mut Diagnostics) -> Result<(), DynError>,
    name: &'static str,
}

impl PassInfo {
    fn get<P: Pass>() -> Self {
        Self {
            assumptions_made: P::ASSUMPTIONS_MADE,
            assumptions_released: P::ASSUMPTIONS_RELEASED,
            pass: Self::run_pass::<P>,
            name: type_name::<P>(),
        }
    }

    fn run_pass<P: Pass>(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<(), DynError> {
        let removals = P::run_pass(manifest, diagnostics)
            .with_message(|| format!("could not finish {} MIR pass", type_name::<P>()))?;
        crate::remove_objects(manifest, removals);
        Ok(())
    }
}

/// Checks if the assumptions hold for the given pass infos (and their order).
/// If not, then Err is returned info about the pass that failed.
fn check_assumptions(passes: &[PassInfo]) -> Result<(), FailedPass> {
    let mut released_assumptions = HashSet::new();

    for pass in passes {
        // Check if all assumptions made are released
        for made_assumption in pass.assumptions_made {
            if !released_assumptions.contains(made_assumption) {
                return Err(FailedPass {
                    pass_name: pass.name,
                    unheld_assumption: *made_assumption,
                });
            }
        }

        // This pass held, release the assumptions for later passes
        for released_assumption in pass.assumptions_released {
            released_assumptions.insert(*released_assumption);
        }
    }

    Ok(())
}

fn randomize_passes(passes: &[PassInfo]) -> Vec<PassInfo> {
    let mut randomized_passes = Vec::new();
    let mut unused_passes = passes.to_vec();
    let mut released_assumptions = HashSet::new();

    while !unused_passes.is_empty() {
        let valid_passes = unused_passes
            .iter()
            .filter(|pass| {
                pass.assumptions_made
                    .iter()
                    .all(|made_assumption| released_assumptions.contains(made_assumption))
            })
            .collect::<Vec<_>>();

        if valid_passes.is_empty() {
            panic!("no valid passes left");
        }

        let chosen_pass = valid_passes[fastrand::usize(0..valid_passes.len())];
        let chosen_pass_index = unused_passes.element_offset(chosen_pass).unwrap();
        let chosen_pass = unused_passes.remove(chosen_pass_index);

        for released_assumption in chosen_pass.assumptions_released {
            released_assumptions.insert(*released_assumption);
        }

        randomized_passes.push(chosen_pass);
    }

    randomized_passes
}

#[derive(Debug)]
struct FailedPass {
    pass_name: &'static str,
    unheld_assumption: Assumption,
}

impl Display for FailedPass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "pass `{}` makes assumption `{:?}`, but that assumption hasn't been released yet",
            self.pass_name, self.unheld_assumption
        )
    }
}
impl Error for FailedPass {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_assumptions_correct() {
        check_assumptions(&[
            PassInfo::get::<DeviceConfigsOwned>(),
            PassInfo::get::<AddressTypesSpecified>(),
        ])
        .unwrap();
        check_assumptions(&[
            PassInfo::get::<AddressTypesSpecified>(),
            PassInfo::get::<DeviceConfigsOwned>(),
        ])
        .unwrap_err();
    }

    #[test]
    fn randomize_ok() {
        for _ in 0..100 {
            let passes = randomize_passes(&get_default_passes());
            check_assumptions(&passes).unwrap();
        }
    }
}
