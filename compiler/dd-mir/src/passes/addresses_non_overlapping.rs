use device_driver_common::specifiers::{Repeat, RepeatSource};

use crate::{
    model::{Device, Manifest, Object, Unique, UniqueId},
    search_object,
};
use device_driver_diagnostics::{Diagnostics, errors::AddressOverlap};

/// Checks if object addresses aren't overlapping when not allowed
pub fn run_pass(manifest: &mut Manifest, diagnostics: &mut Diagnostics) {
    for (device, _) in manifest.iter_devices_with_config() {
        let register_addresses = find_object_addresses(manifest, device, |o| {
            matches!(o, Object::Block(_) | Object::Register(_))
        });
        check_for_overlap(&register_addresses, diagnostics);
        let command_addresses = find_object_addresses(manifest, device, |o| {
            matches!(o, Object::Block(_) | Object::Command(_))
        });
        check_for_overlap(&command_addresses, diagnostics);
        let buffer_addresses = find_object_addresses(manifest, device, |o| {
            matches!(o, Object::Block(_) | Object::Buffer(_))
        });
        check_for_overlap(&buffer_addresses, diagnostics);
    }
}

fn check_for_overlap(addresses: &[ObjectAddress], diagnostics: &mut Diagnostics) {
    for (i, address) in addresses.iter().enumerate() {
        for check_address in addresses.iter().skip(i + 1) {
            if address.address == check_address.address
                && (!address.allow_overlap || !check_address.allow_overlap)
            {
                diagnostics.add(AddressOverlap {
                    address: address.address,
                    object_1: address.id.span(),
                    repeat_offset_1: address.repeat_offset,
                    object_2: check_address.id.span(),
                    repeat_offset_2: check_address.repeat_offset,
                });
            }
        }
    }
}

struct ObjectAddress {
    id: UniqueId,
    // Address including repeat offset
    address: i128,
    repeat_offset: Option<i128>,
    allow_overlap: bool,
}

fn find_object_addresses<'m>(
    manifest: &'m Manifest,
    device: &'m Device,
    filter: impl Fn(&'m Object) -> bool,
) -> Vec<ObjectAddress> {
    let mut object_addresses = Vec::new();

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

        // Only non-block and non-device objects are cared about for the addresses
        if let Some(address) = object.address()
            && matches!(
                object,
                Object::Register(_) | Object::Command(_) | Object::Buffer(_)
            )
        {
            let repeat = object.repeat().cloned().unwrap_or(Repeat {
                source: RepeatSource::Count(1),
                stride: 0,
            });

            let total_address_offsets = address_offsets.iter().sum::<i128>();

            match repeat.source {
                RepeatSource::Count(count) => {
                    for index in 0..i128::from(count) {
                        let repeat_offset = index * repeat.stride;
                        let address = total_address_offsets + address.value + repeat_offset;

                        object_addresses.push(ObjectAddress {
                            id: object.id(),
                            address,
                            repeat_offset: object.repeat().map(|_| repeat_offset),
                            allow_overlap: object.allow_address_overlap(),
                        });
                    }
                }
                RepeatSource::Enum(enum_name) => {
                    let enum_value = search_object(manifest, &enum_name)
                        .expect("A mir pass checked this enum exists")
                        .as_enum()
                        .expect("A mir pass checked this is an enum");

                    for (discriminant, _) in enum_value.iter_variants_with_discriminant() {
                        let repeat_offset = discriminant * repeat.stride;
                        let address = total_address_offsets + address.value + repeat_offset;

                        object_addresses.push(ObjectAddress {
                            id: object.id(),
                            address,
                            repeat_offset: Some(repeat_offset),
                            allow_overlap: object.allow_address_overlap(),
                        });
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

    object_addresses
}
