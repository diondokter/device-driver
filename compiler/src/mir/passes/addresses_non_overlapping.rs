use crate::{
    mir::{
        Device, Manifest, Object, Repeat, RepeatSource, Unique, UniqueId, passes::search_object,
    },
    reporting::Diagnostics,
};

/// Checks if object addresses aren't overlapping when not allowed
pub fn run_pass(manifest: &mut Manifest, diagnostics: &mut Diagnostics) {
    for (device, _) in manifest.iter_devices_with_config() {
        let register_addresses = find_object_adresses(manifest, device, |o| {
            matches!(o, Object::Block(_) | Object::Register(_))
        });
        check_for_overlap(&register_addresses, diagnostics);
        let command_addresses = find_object_adresses(manifest, device, |o| {
            matches!(o, Object::Block(_) | Object::Command(_))
        });
        check_for_overlap(&command_addresses, diagnostics);
        let buffer_addresses = find_object_adresses(manifest, device, |o| {
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
                diagnostics.add_msg(format!("Overlap between {} (repeat offset {:?}) & {} (repeat offset {:?}) at address {}", address.id, address.repeat_offset, check_address.id, check_address.repeat_offset, address.address));
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

fn find_object_adresses<'m>(
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
            && object.child_objects().is_empty()
        {
            let repeat = object.repeat().cloned().unwrap_or(Repeat {
                source: RepeatSource::Count(1),
                stride: 0,
            });

            let total_address_offsets = address_offsets.iter().sum::<i128>();

            match repeat.source {
                RepeatSource::Count(count) => {
                    for index in 0..count as i128 {
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
