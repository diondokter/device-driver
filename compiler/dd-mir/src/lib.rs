use device_driver_common::{
    identifier::IdentifierRef,
    specifiers::{Repeat, RepeatSource},
};
use device_driver_diagnostics::Diagnostics;
use device_driver_parser::Ast;

use crate::model::{Device, Manifest, Object};

mod lowering;
pub mod model;
pub(crate) mod passes;

pub fn lower_ast(ast: Ast, diagnostics: &mut Diagnostics) -> model::Manifest {
    let mut mir = lowering::transform(ast, diagnostics);

    passes::run_passes(&mut mir, diagnostics);

    mir
}

pub fn search_object<'o>(manifest: &'o Manifest, name: &IdentifierRef) -> Option<&'o Object> {
    manifest
        .iter_objects()
        .find(|o| o.name().original() == name.original())
}

/// Returns None if device has no objects that pass the filter
#[expect(clippy::type_complexity, reason = "I disagree")]
pub fn find_min_max_addresses<'m>(
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
                        count_0_address + (i128::from(count.saturating_sub(1)) * repeat.stride);
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
