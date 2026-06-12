use std::{collections::HashSet, num::NonZero};

use clap::Parser;
use device_driver_common::{
    identifier::{IdentifierRef, IdentifierType},
    span::SpanExt,
    specifiers::{Repeat, RepeatSource},
};
use device_driver_diagnostics::{Diagnostics, DynError};
use device_driver_parser::Ast;

use crate::model::{Device, LendingIterator, Manifest, Object, Unique, UniqueId};

mod lowering;
pub mod model;
pub(crate) mod passes;

#[derive(Parser, Debug, Clone, Default)]
#[command(no_binary_name = true)]
pub struct MirOptions {
    /// The seed to use for randomization. If not specified, a random seed is used
    #[arg(
        long = "unstable-mir-randomize-seed",
        require_equals = true,
        global = true
    )]
    pub randomize_mir_passes_seed: Option<u64>,
    /// Randomize the order of the mir passes
    #[arg(long = "unstable-mir-randomize-passes", global = true)]
    pub randomize_mir_passes: bool,
    /// Run assumption checks for the passes
    #[arg(long = "unstable-mir-check-assumptions", global = true)]
    pub check_assumptions: bool,
}

pub fn lower_ast(
    ast: Ast,
    options: MirOptions,
    diagnostics: &mut Diagnostics,
) -> Result<model::Manifest, DynError> {
    let mut mir = lowering::lower(ast, diagnostics);

    passes::run_passes(&mut mir, options, diagnostics)?;

    Ok(mir)
}

pub fn search_object<'o, T: IdentifierType>(
    manifest: &'o Manifest,
    name: &IdentifierRef<T>,
) -> Option<&'o Object> {
    manifest.iter_objects().find(|o| name.is_ref_to(o.name()))
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
                source: RepeatSource::Count(NonZero::new(1).unwrap()),
                stride: 0.with_dummy_span(),
            });

            let total_address_offsets = address_offsets.iter().sum::<i128>();

            match repeat.source {
                RepeatSource::Count(count) => {
                    let count_0_address = total_address_offsets + address.value;
                    let count_max_address = count_0_address
                        + (i128::from(count.get().saturating_sub(1)) * repeat.stride.value);
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
                        let address = total_address_offsets
                            + address.value
                            + (discriminant * repeat.stride.value);
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
