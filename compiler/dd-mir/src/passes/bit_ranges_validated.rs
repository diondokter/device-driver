use std::collections::HashSet;

use device_driver_common::specifiers::{AddressRange, RepeatSource};

use crate::{
    model::{Field, FieldSet, Manifest, ObjectId},
    passes::{Assumption, Pass},
};
use device_driver_diagnostics::{
    Diagnostics, DynError,
    errors::{FieldAddressExceedsFieldsetSize, FieldAddressNegative, OverlappingFields},
};

/// Validate that the bit ranges of fields fall within the max size and don't have overlap if they're not allowed
pub struct BitRangesValidated;

impl Pass for BitRangesValidated {
    const ASSUMPTIONS_MADE: &[Assumption] = &[
        Assumption::RepeatStrideNonZero,
        Assumption::RepeatEnumRefValid,
        Assumption::NamesUnique,
        Assumption::LocalNamespacesAssigned,
    ];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[];

    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<ObjectId>, DynError> {
        let mut removals = HashSet::new();

        for fieldset in manifest.fieldsets.iter() {
            validate_len(fieldset, manifest, diagnostics, &mut removals);
            if !fieldset.allow_bit_overlap {
                validate_overlap(fieldset, manifest, diagnostics);
            }
        }

        Ok(removals)
    }
}

fn validate_len(
    field_set: &FieldSet,
    manifest: &Manifest,
    diagnostics: &mut Diagnostics,
    removals: &mut HashSet<ObjectId>,
) {
    for field_id in &field_set.fields {
        let field = manifest.fields.get(*field_id).unwrap();

        let field_len = field.field_address.len();

        if field_len == 0 {
            panic!("A zero-sized field can't be specified");
        }

        let (offset_iter, repeated) = get_repeat_iter(manifest, field);

        let max_repeat_offset = offset_iter.iter().max().unwrap();
        let min_repeat_offset = offset_iter.iter().min().unwrap();

        let max_field_end = i128::from(field.field_address.end) + max_repeat_offset;
        let min_field_start = i128::from(field.field_address.start) + min_repeat_offset;

        if max_field_end >= i128::from(field_set.size_bits()) {
            diagnostics.add(FieldAddressExceedsFieldsetSize {
                address: field.field_address.span,
                max_field_end,
                repeat_offset: repeated.then_some(*max_repeat_offset),
                fieldset_size_bits: field_set.size_bits(),
                fieldset_size_span: field_set.size_bytes.span,
            });
            removals.insert((*field_id).into());
        }

        if min_field_start < 0 {
            diagnostics.add(FieldAddressNegative {
                address: field.field_address.span,
                min_field_start,
                repeat_offset: repeated.then_some(*min_repeat_offset),
                field_set_context: field_set.name.span,
            });
            removals.insert((*field_id).into());
        }
    }
}

fn validate_overlap(field_set: &FieldSet, manifest: &Manifest, diagnostics: &mut Diagnostics) {
    for (i, field_id) in field_set.fields.iter().enumerate() {
        let field = manifest.fields.get(*field_id).unwrap();
        let (offsets, repeated) = get_repeat_iter(manifest, field);

        'second_field: for second_field_id in field_set.fields.iter().skip(i + 1) {
            let second_field = manifest.fields.get(*second_field_id).unwrap();
            let (second_offsets, second_repeated) = get_repeat_iter(manifest, second_field);

            for offset in &offsets {
                for second_offset in &second_offsets {
                    if ranges_overlap(
                        &field.field_address,
                        *offset,
                        &second_field.field_address,
                        *second_offset,
                    ) {
                        diagnostics.add(OverlappingFields {
                            field_address_1: field.field_address.span,
                            repeat_offset_1: repeated.then_some(*offset),
                            field_address_start_1: i128::from(field.field_address.start) + offset,
                            field_address_end_1: i128::from(field.field_address.end) + offset,
                            field_address_2: second_field.field_address.span,
                            repeat_offset_2: second_repeated.then_some(*second_offset),
                            field_address_start_2: i128::from(second_field.field_address.start)
                                + second_offset,
                            field_address_end_2: i128::from(second_field.field_address.end)
                                + second_offset,

                            field_set_context: field_set.name.span,
                        });

                        continue 'second_field;
                    }
                }
            }
        }
    }
}

fn ranges_overlap(l: &AddressRange, offset: i128, r: &AddressRange, second_offset: i128) -> bool {
    (i128::from(l.start) + offset) <= (i128::from(r.end) + second_offset)
        && (i128::from(r.start) + second_offset) <= (i128::from(l.end) + offset)
}

fn get_repeat_iter(manifest: &Manifest, field: &Field) -> (Vec<i128>, bool) {
    if let Some(repeat) = &field.repeat {
        let stride = repeat.stride;
        match &repeat.source.value {
            RepeatSource::Count(count) => (
                (0..i128::from(count.get()))
                    .map(move |count| count * stride.value)
                    .collect(),
                true,
            ),
            RepeatSource::Enum(enum_name) => (
                manifest
                    .search_object(enum_name)
                    .expect("Checked in earlier pass")
                    .as_enum()
                    .expect("Checked in earlier pass")
                    .iter_variants_with_discriminant(manifest)
                    .map(move |(discriminant, _)| discriminant * stride.value)
                    .collect(),
                true,
            ),
        }
    } else {
        (vec![0], false)
    }
}
