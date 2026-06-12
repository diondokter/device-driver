use std::collections::HashSet;

use device_driver_common::{span::Spanned, specifiers::Integer};
use device_driver_diagnostics::{Diagnostics, DynError, errors::AddressOutOfRange};

use crate::{
    find_min_max_addresses,
    model::{Device, Manifest, Object, Unique, UniqueId},
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
    ];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[];

    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<UniqueId>, DynError> {
        let mut removals = HashSet::new();

        for object in manifest.iter_objects() {
            let Object::Device(device) = object else {
                continue;
            };

            check_device(
                device.device_config.register_address_type.as_ref(),
                manifest,
                device,
                |o| matches!(o, Object::Block(_) | Object::Register(_)),
                diagnostics,
                &mut removals,
            );
            check_device(
                device.device_config.command_address_type.as_ref(),
                manifest,
                device,
                |o| matches!(o, Object::Block(_) | Object::Command(_)),
                diagnostics,
                &mut removals,
            );
            check_device(
                device.device_config.buffer_address_type.as_ref(),
                manifest,
                device,
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
    filter: impl Fn(&Object) -> bool,
    diagnostics: &mut Diagnostics,
    removals: &mut HashSet<UniqueId>,
) {
    let Some(address_type) = address_type else {
        return;
    };

    let Some(((min_address, min_obj), (max_address, _))) =
        find_min_max_addresses(manifest, device, filter)
    else {
        return;
    };

    if min_address < address_type.min_value() || max_address > address_type.max_value() {
        diagnostics.add(AddressOutOfRange {
            object: min_obj.name_span(),
            address: min_obj
                .address()
                .expect("All objects here should have addresses")
                .span,
            address_value_min: min_address,
            address_value_max: max_address,
            address_type_config: address_type.span,
            address_type: address_type.value,
        });
        removals.insert(device.id());
    }
}

#[cfg(test)]
mod tests {
    use device_driver_common::{
        identifier::Identifier,
        span::{Span, SpanExt},
        specifiers::Integer,
    };

    use crate::model::{Command, Device, DeviceConfig, Register};

    use super::*;

    #[test]
    fn not_too_low() {
        let mut start_mir = Device {
            description: String::new(),
            name: Identifier::try_parse("Device").unwrap().with_dummy_span(),
            device_config: DeviceConfig {
                register_address_type: Some(Integer::I8.with_dummy_span()),
                ..Default::default()
            },
            objects: vec![Object::Register(Register {
                name: Identifier::try_parse("MyReg").unwrap().with_dummy_span(),
                address: (-300).with_dummy_span(),
                ..Default::default()
            })],
            span: Span::default(),
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        assert!(
            !AddressTypesBigEnough::run_pass(&mut start_mir, &mut diagnostics)
                .unwrap()
                .is_empty()
        );
        assert!(diagnostics.has_error());
    }

    #[test]
    fn not_too_high() {
        let mut start_mir = Device {
            description: String::new(),
            name: Identifier::try_parse("Device").unwrap().with_dummy_span(),
            device_config: DeviceConfig {
                command_address_type: Some(Integer::U16.with_dummy_span()),
                ..Default::default()
            },
            objects: vec![Object::Command(Command {
                name: Identifier::try_parse("MyReg").unwrap().with_dummy_span(),
                address: 128000.with_dummy_span(),
                ..Default::default()
            })],
            span: Span::default(),
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        assert!(
            !AddressTypesBigEnough::run_pass(&mut start_mir, &mut diagnostics)
                .unwrap()
                .is_empty()
        );
        assert!(diagnostics.has_error());
    }
}
