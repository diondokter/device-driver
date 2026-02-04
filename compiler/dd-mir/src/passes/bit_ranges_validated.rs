use std::{collections::HashSet, ops::Range};

use device_driver_common::specifiers::RepeatSource;

use crate::{
    model::{Field, FieldSet, Manifest, Unique, UniqueId},
    search_object,
};
use device_driver_diagnostics::{
    Diagnostics,
    errors::{
        FieldAddressExceedsFieldsetSize, FieldAddressNegative, OverlappingFields, ZeroSizeField,
    },
};

/// Validate that the bit ranges of fields fall within the max size and don't have overlap if they're not allowed
pub fn run_pass(manifest: &mut Manifest, diagnostics: &mut Diagnostics) -> HashSet<UniqueId> {
    let mut removals = HashSet::new();

    for object in manifest.iter_objects() {
        if let Some(field_set) = object.as_field_set() {
            validate_len(field_set, manifest, diagnostics, &mut removals);
            if !field_set.allow_bit_overlap {
                validate_overlap(field_set, manifest, diagnostics);
            }
        }
    }

    removals
}

fn validate_len(
    field_set: &FieldSet,
    manifest: &Manifest,
    diagnostics: &mut Diagnostics,
    removals: &mut HashSet<UniqueId>,
) {
    for field in &field_set.fields {
        let field_len = field.field_address.value.clone().count();

        if field_len == 0 {
            diagnostics.add(ZeroSizeField {
                address: field.field_address.span,
                address_bits: field_len as u32,
            });
        }

        let (offset_iter, repeated) = get_repeat_iter(manifest, field);

        let max_repeat_offset = offset_iter.iter().max().unwrap();
        let min_repeat_offset = offset_iter.iter().min().unwrap();

        let max_field_end = i128::from(field.field_address.end) + max_repeat_offset;
        let min_field_start = i128::from(field.field_address.start) + min_repeat_offset;

        if max_field_end > i128::from(field_set.size_bits.value) {
            diagnostics.add(FieldAddressExceedsFieldsetSize {
                address: field.field_address.span,
                max_field_end: max_field_end - 1,
                repeat_offset: repeated.then_some(*max_repeat_offset),
                fieldset_size: field_set.size_bits.value,
                fieldset_size_bits: field_set.size_bits.span,
            });
            removals.insert(field.id_with(field_set.id()));
        }

        if min_field_start < 0 {
            diagnostics.add(FieldAddressNegative {
                address: field.field_address.span,
                min_field_start,
                repeat_offset: repeated.then_some(*min_repeat_offset),
            });
            removals.insert(field.id_with(field_set.id()));
        }
    }
}

fn validate_overlap(field_set: &FieldSet, manifest: &Manifest, diagnostics: &mut Diagnostics) {
    for (i, field) in field_set.fields.iter().enumerate() {
        let (offsets, repeated) = get_repeat_iter(manifest, field);

        'second_field: for second_field in field_set.fields.iter().skip(i + 1) {
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
                        });

                        continue 'second_field;
                    }
                }
            }
        }
    }
}

fn ranges_overlap(l: &Range<u32>, offset: i128, r: &Range<u32>, second_offset: i128) -> bool {
    (i128::from(l.start) + offset) < (i128::from(r.end) + second_offset)
        && (i128::from(r.start) + second_offset) < (i128::from(l.end) + offset)
}

fn get_repeat_iter(manifest: &Manifest, field: &Field) -> (Vec<i128>, bool) {
    if let Some(repeat) = &field.repeat {
        let stride = repeat.stride;
        match &repeat.source {
            RepeatSource::Count(count) => (
                (0..i128::from(*count))
                    .map(move |count| count * stride)
                    .collect(),
                true,
            ),
            RepeatSource::Enum(enum_name) => (
                search_object(manifest, enum_name)
                    .expect("Checked in earlier pass")
                    .as_enum()
                    .expect("Checked in earlier pass")
                    .iter_variants_with_discriminant()
                    .map(move |(discriminant, _)| discriminant * stride)
                    .collect(),
                true,
            ),
        }
    } else {
        (vec![0], false)
    }
}

#[cfg(test)]
mod tests {
    use device_driver_common::{span::SpanExt, specifiers::Repeat};

    use crate::model::{Device, Field, Object};

    use super::*;

    #[test]
    fn max_len_exceeded() {
        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".into_with_dummy_span(),
                size_bits: 10.with_dummy_span(),
                fields: vec![Field {
                    name: "my_field".into_with_dummy_span(),
                    field_address: (0..10).with_dummy_span(),
                    ..Default::default()
                }],
                ..Default::default()
            })],
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
        assert!(!diagnostics.has_error());

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".into_with_dummy_span(),
                size_bits: 10.with_dummy_span(),
                fields: vec![Field {
                    name: "my_field".into_with_dummy_span(),
                    field_address: (0..11).with_dummy_span(),
                    ..Default::default()
                }],
                ..Default::default()
            })],
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
        assert!(diagnostics.has_error());

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".into_with_dummy_span(),
                size_bits: 10.with_dummy_span(),
                fields: vec![Field {
                    name: "my_field".into_with_dummy_span(),
                    field_address: (0..10).with_dummy_span(),
                    ..Default::default()
                }],
                ..Default::default()
            })],
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
        assert!(!diagnostics.has_error());

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".into_with_dummy_span(),
                size_bits: 10.with_dummy_span(),
                fields: vec![Field {
                    name: "my_field".into_with_dummy_span(),
                    field_address: (0..11).with_dummy_span(),
                    ..Default::default()
                }],
                ..Default::default()
            })],
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
        assert!(diagnostics.has_error());

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".into_with_dummy_span(),
                size_bits: 10.with_dummy_span(),
                fields: vec![Field {
                    name: "my_field".into_with_dummy_span(),
                    field_address: (0..10).with_dummy_span(),
                    ..Default::default()
                }],
                ..Default::default()
            })],
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
        assert!(!diagnostics.has_error());

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".into_with_dummy_span(),
                size_bits: 10.with_dummy_span(),
                fields: vec![Field {
                    name: "my_field".into_with_dummy_span(),
                    field_address: (0..11).with_dummy_span(),
                    ..Default::default()
                }],
                ..Default::default()
            })],
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
        assert!(diagnostics.has_error());

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".into_with_dummy_span(),
                size_bits: 10.with_dummy_span(),
                fields: vec![Field {
                    name: "my_field".into_with_dummy_span(),
                    field_address: (0..5).with_dummy_span(),
                    repeat: Some(Repeat {
                        source: RepeatSource::Count(3),
                        stride: 5,
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            })],
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
        assert!(diagnostics.has_error());
    }

    #[test]
    fn overlap() {
        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".into_with_dummy_span(),
                size_bits: 10.with_dummy_span(),
                fields: vec![
                    Field {
                        name: "my_field".into_with_dummy_span(),
                        field_address: (0..5).with_dummy_span(),
                        ..Default::default()
                    },
                    Field {
                        name: "my_field2".into_with_dummy_span(),
                        field_address: (5..10).with_dummy_span(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            })],
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
        assert!(!diagnostics.has_error());

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".into_with_dummy_span(),
                size_bits: 10.with_dummy_span(),
                allow_bit_overlap: true,
                fields: vec![
                    Field {
                        name: "my_field".into_with_dummy_span(),
                        field_address: (0..6).with_dummy_span(),
                        ..Default::default()
                    },
                    Field {
                        name: "my_field2".into_with_dummy_span(),
                        field_address: (5..10).with_dummy_span(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            })],
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
        assert!(!diagnostics.has_error());

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".into_with_dummy_span(),
                size_bits: 10.with_dummy_span(),
                fields: vec![
                    Field {
                        name: "my_field".into_with_dummy_span(),
                        field_address: (0..6).with_dummy_span(),
                        ..Default::default()
                    },
                    Field {
                        name: "my_field2".into_with_dummy_span(),
                        field_address: (5..10).with_dummy_span(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            })],
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
        assert!(diagnostics.has_error());

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".into_with_dummy_span(),
                size_bits: 10.with_dummy_span(),
                fields: vec![
                    Field {
                        name: "my_field".into_with_dummy_span(),
                        field_address: (0..1).with_dummy_span(),
                        repeat: Some(Repeat {
                            source: RepeatSource::Count(6),
                            stride: 1,
                        }),
                        ..Default::default()
                    },
                    Field {
                        name: "my_field2".into_with_dummy_span(),
                        field_address: (5..10).with_dummy_span(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            })],
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
        assert!(diagnostics.has_error());
    }
}
