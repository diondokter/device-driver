use std::collections::HashSet;

use device_driver_common::{span::Spanned, specifiers::Integer};
use device_driver_diagnostics::{Diagnostics, DynError, errors::AddressOutOfRange};

use crate::{
    find_min_max_addresses,
    model::{Device, DeviceId, Manifest, Object, ObjectId},
    passes::{Assumption, Pass},
};

/// Checks if the various address types can fully contain the min and max addresses of the types of objects they are for
pub struct AddressTypesBigEnough;

impl Pass for AddressTypesBigEnough {
    const ASSUMPTIONS_MADE: &[Assumption] = &[
        Assumption::AddressTypesSpecified,
        Assumption::RepeatStrideNonZero,
        Assumption::NamesUnique,
        Assumption::RepeatEnumRefValid,
        Assumption::RepeatMathChecked,
    ];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[];

    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<ObjectId>, DynError> {
        let mut removals = HashSet::new();

        for (id, device) in manifest.devices.iter_enumerated() {
            check_device(
                device.device_config.register_address_type.as_ref(),
                manifest,
                device,
                id,
                |o| matches!(o, Object::Block(_) | Object::Register(_)),
                diagnostics,
                &mut removals,
            );
            check_device(
                device.device_config.command_address_type.as_ref(),
                manifest,
                device,
                id,
                |o| matches!(o, Object::Block(_) | Object::Command(_)),
                diagnostics,
                &mut removals,
            );
            check_device(
                device.device_config.buffer_address_type.as_ref(),
                manifest,
                device,
                id,
                |o| matches!(o, Object::Block(_) | Object::Buffer(_)),
                diagnostics,
                &mut removals,
            );
        }

        Ok(removals)
    }
}

fn check_device(
    address_type: Option<&Spanned<Integer>>,
    manifest: &Manifest,
    device: &Device,
    id: DeviceId,
    filter: impl Fn(Object) -> bool,
    diagnostics: &mut Diagnostics,
    removals: &mut HashSet<ObjectId>,
) {
    let Some(address_type) = address_type else {
        return;
    };

    let Some(((min_address, min_obj), (max_address, max_obj))) =
        find_min_max_addresses(manifest, device, filter)
    else {
        return;
    };

    if min_address < address_type.min_value() || max_address > address_type.max_value() {
        let diagnostic_object = if max_address > address_type.max_value() {
            max_obj
        } else {
            min_obj
        };

        diagnostics.add(AddressOutOfRange {
            object: diagnostic_object.name_span(),
            address: diagnostic_object
                .address()
                .expect("All objects here should have addresses")
                .span,
            address_value_min: min_address,
            address_value_max: max_address,
            address_type_config: address_type.span,
            address_type: address_type.value,
        });
        removals.insert(id.into());
    }
}
