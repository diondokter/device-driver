use std::{num::NonZero, time::Duration};

use clap::Parser;
use device_driver_common::{
    span::{Span, SpanExt},
    specifiers::{Repeat, RepeatSource},
};
use device_driver_diagnostics::{Diagnostics, DynError};
use device_driver_parser::Ast;

use crate::model::{Device, Manifest, Object};

mod lowering;
pub mod model;
pub(crate) mod passes;

#[cfg(feature = "gen-docs")]
pub use lowering::gen_docs::gen_docs;

#[derive(Parser, Debug, Clone, Default)]
#[command(no_binary_name = true)]
pub struct MirOptions {
    /// The seed to use for randomization. If not specified, a random seed is used
    #[arg(
        long = "unstable-mir-randomize-seed",
        require_equals = true,
        global = true,
        value_name = "SEED"
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
    ast: &Ast,
    options: &MirOptions,
    diagnostics: &mut Diagnostics,
) -> Result<(model::Manifest, Vec<PassTiming>), DynError> {
    let mut mir = lowering::lower(ast, diagnostics);

    let pass_timings = passes::run_passes(&mut mir, options, diagnostics)?;

    Ok((mir, pass_timings))
}

/// Returns None if device has no objects that pass the filter
///
/// This assumes [passes::Assumption::RepeatStrideNonZero], [passes::Assumption::NamesUnique] & [passes::Assumption::RepeatEnumRefValid]
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

    let mut children_left = vec![device.children.len()];
    let mut address_offsets = vec![device.address_offset.value];

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
                source: RepeatSource::Count(NonZero::new(1).unwrap()).with_dummy_span(),
                stride: 0.with_dummy_span(),
                span: Span::empty(),
            });

            let total_address_offsets = address_offsets.iter().sum::<i128>();

            match repeat.source.value {
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
                    let enum_value = manifest
                        .search_object(&enum_name)
                        .expect("A mir pass checked this enum exists")
                        .as_enum()
                        .expect("A mir pass checked this is an enum");

                    for (discriminant, _) in enum_value.iter_variants_with_discriminant(manifest) {
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
                address_offsets.push(d.address_offset.value);
                children_left.push(d.children.len());
            }
            Object::Block(b) => {
                address_offsets.push(b.address_offset.value);
                children_left.push(b.children.len());
            }
            _ => (),
        }
    }

    Some((
        (min_address_found, min_obj_found?),
        (max_address_found, max_obj_found?),
    ))
}

#[derive(Debug)]
pub struct PassTiming {
    pub name: String,
    pub duration: Duration,
}
