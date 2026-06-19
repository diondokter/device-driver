use std::{collections::HashSet, num::NonZero};

use device_driver_common::{
    span::{SpanExt, Spanned},
    specifiers::{AddressMode, Repeat, RepeatSource},
};

use crate::{
    model::{Device, DeviceConfig, Manifest, Object, Unique, UniqueId},
    passes::{Assumption, Pass},
    search_object,
};
use device_driver_diagnostics::{Diagnostics, DynError, ResultExt, errors::AddressOverlap};

/// Checks if object addresses aren't overlapping when not allowed
pub struct AddressesNonOverlapping;

impl Pass for AddressesNonOverlapping {
    const ASSUMPTIONS_MADE: &[Assumption] = &[
        Assumption::RepeatStrideNonZero,
        Assumption::RepeatEnumRefValid,
        Assumption::NamesUnique,
        Assumption::FieldsetRefsValid,
    ];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[];

    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<UniqueId>, DynError> {
        for (device, config) in manifest.iter_devices_with_config() {
            let register_addresses = find_object_addresses(manifest, device, &config, |o| {
                matches!(o, Object::Block(_) | Object::Register(_))
            })
            .with_message(|| "finding register object addresses")?;
            check_for_overlap(&register_addresses, diagnostics);
            let command_addresses = find_object_addresses(manifest, device, &config, |o| {
                matches!(o, Object::Block(_) | Object::Command(_))
            })
            .with_message(|| "finding command object addresses")?;
            check_for_overlap(&command_addresses, diagnostics);
            let buffer_addresses = find_object_addresses(manifest, device, &config, |o| {
                matches!(o, Object::Block(_) | Object::Buffer(_))
            })
            .with_message(|| "finding buffer object addresses")?;
            check_for_overlap(&buffer_addresses, diagnostics);
        }

        Ok(Default::default())
    }
}

fn overlap_point(x1: i128, x2: i128, y1: i128, y2: i128) -> Option<i128> {
    if (x1..x2).is_empty() || (y1..y2).is_empty() {
        None
    } else if (x1..x2).contains(&y1) {
        Some(y1)
    } else if (x1..x2).contains(&(y2 - 1)) {
        Some(y2 - 1)
    } else if (y1..y2).contains(&x1) {
        Some(x1)
    } else if (y1..y2).contains(&(x2 - 1)) {
        Some(x2 - 1)
    } else {
        None
    }
}

fn check_for_overlap(addresses: &[ObjectAddress], diagnostics: &mut Diagnostics) {
    for (i, address) in addresses.iter().enumerate() {
        for check_address in addresses.iter().skip(i + 1) {
            let address_start = address.address.value;
            let address_end = address_start + address.size.value as i128;

            let check_address_start = check_address.address.value;
            let check_address_end = check_address_start + check_address.size.value as i128;

            if let Some(overlap_point) = overlap_point(
                address_start,
                address_end,
                check_address_start,
                check_address_end,
            ) && (!address.allow_overlap || !check_address.allow_overlap)
            {
                diagnostics.add(AddressOverlap {
                    address: overlap_point,
                    object_1: address.id.span(),
                    object_1_address: address.address.span,
                    object_1_size: address.size.span,
                    repeat_offset_1: address.repeat_offset,
                    object_2: check_address.id.span(),
                    object_2_address: check_address.address.span,
                    object_2_size: check_address.size.span,
                    repeat_offset_2: check_address.repeat_offset,
                });
            }
        }
    }
}

struct ObjectAddress {
    id: UniqueId,
    // Address including repeat offset
    address: Spanned<i128>,
    size: Spanned<u32>,
    repeat_offset: Option<i128>,
    allow_overlap: bool,
}

fn find_object_addresses<'m>(
    manifest: &'m Manifest,
    device: &'m Device,
    config: &DeviceConfig,
    filter: impl Fn(&'m Object) -> bool,
) -> Result<Vec<ObjectAddress>, DynError> {
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
            let size = if matches!(
                config.register_address_mode.map(|s| s.value),
                Some(AddressMode::Mapped)
            ) && let Object::Register(object) = object
            {
                let Some(Object::FieldSet(fs)) =
                    search_object(manifest, &object.field_set_ref.value)
                else {
                    return Err(DynError::new(format!(
                        "returned object for `{}` is none or not a fieldset, but it was safe to assume it would be",
                        object.field_set_ref.original()
                    )));
                };

                fs.size_bytes
            } else {
                1.with_dummy_span()
            };

            let repeat = object.repeat().cloned().unwrap_or(Repeat {
                source: RepeatSource::Count(NonZero::new(1).unwrap()),
                stride: 0.with_dummy_span(),
            });

            let total_address_offsets = address_offsets.iter().sum::<i128>();

            // If the stride is 0, everything overlaps. We don't need infinite diagnostics about that,
            // so limit the elements we look at. Otherwise we could OOM
            let max_elements = if repeat.stride == 0 { 5 } else { usize::MAX };

            match repeat.source {
                RepeatSource::Count(count) => {
                    for index in (0..i128::from(count.get())).take(max_elements) {
                        let repeat_offset = index * repeat.stride.value;
                        let address_value = total_address_offsets + address.value + repeat_offset;

                        object_addresses.push(ObjectAddress {
                            id: object.id(),
                            address: address_value.with_span(address.span),
                            size,
                            repeat_offset: object.repeat().map(|_| repeat_offset),
                            allow_overlap: object.allow_address_overlap(),
                        });
                    }
                }
                RepeatSource::Range { end, start } => {
                    for index in (start..=end).take(max_elements) {
                        let repeat_offset = index * repeat.stride.value;
                        let address_value = total_address_offsets + address.value + repeat_offset;

                        object_addresses.push(ObjectAddress {
                            id: object.id(),
                            address: address_value.with_span(address.span),
                            size,
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

                    for (discriminant, _) in enum_value
                        .iter_variants_with_discriminant()
                        .take(max_elements)
                    {
                        let repeat_offset = discriminant * repeat.stride.value;
                        let address_value = total_address_offsets + address.value + repeat_offset;

                        object_addresses.push(ObjectAddress {
                            id: object.id(),
                            address: address_value.with_span(address.span),
                            size,
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

    Ok(object_addresses)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlap_point() {
        std::assert_matches!(overlap_point(0, 1, 1, 2), None);
        std::assert_matches!(overlap_point(0, 2, 1, 3), Some(1));
    }
}
