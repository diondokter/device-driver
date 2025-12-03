use std::{collections::HashSet, ops::Range};

use miette::ensure;

use crate::{
    mir::{FieldSet, Manifest, RepeatSource, Unique, UniqueId},
    reporting::{
        Diagnostics,
        errors::{FieldAddressExceedsFieldsetSize, FieldAddressNegative, ZeroSizeField},
    },
};

/// Validate that the bit ranges of fields fall within the max size and don't have overlap if they're not allowed
pub fn run_pass(
    manifest: &mut Manifest,
    diagnostics: &mut Diagnostics,
) -> miette::Result<HashSet<UniqueId>> {
    let mut removals = HashSet::new();

    for object in manifest.iter_objects() {
        if let Some(field_set) = object.as_field_set() {
            validate_len(field_set, manifest, diagnostics, &mut removals)?;
            if !field_set.allow_bit_overlap {
                validate_overlap(field_set, manifest)?;
            }
        }
    }

    Ok(removals)
}

fn validate_len(
    field_set: &FieldSet,
    manifest: &Manifest,
    diagnostics: &mut Diagnostics,
    removals: &mut HashSet<UniqueId>,
) -> miette::Result<()> {
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

        let max_field_end = field.field_address.end as i128 + max_repeat_offset;
        let min_field_start = field.field_address.start as i128 + min_repeat_offset;

        if max_field_end > field_set.size_bits.value as i128 {
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

    Ok(())
}

fn validate_overlap(field_set: &FieldSet, manifest: &Manifest) -> miette::Result<()> {
    for (i, field) in field_set.fields.iter().enumerate() {
        let (offsets, repeated) = get_repeat_iter(manifest, field);

        for (j, second_field) in field_set.fields.iter().enumerate() {
            let (second_offsets, second_repeated) = get_repeat_iter(manifest, second_field);

            for offset in offsets.iter() {
                let repeat_info = if repeated {
                    format!(" (at repeat offset: {offset})")
                } else {
                    "".into()
                };

                for second_offset in second_offsets.iter() {
                    let second_repeat_info = if second_repeated {
                        format!(" (at repeat offset: {second_offset})")
                    } else {
                        "".into()
                    };

                    if i == j && offset == second_offset {
                        // No need to compare with self
                        continue;
                    }

                    ensure!(
                        !ranges_overlap(
                            &field.field_address,
                            *offset,
                            &second_field.field_address,
                            *second_offset
                        ),
                        "Fieldset `{}` has two overlapping fields: `{}`{repeat_info} and `{}`{second_repeat_info}. If this is intended, set the `AllowBitOverlap` option to true",
                        field_set.name,
                        field.name,
                        second_field.name
                    )
                }
            }
        }
    }

    Ok(())
}

fn ranges_overlap(l: &Range<u32>, offset: i128, r: &Range<u32>, second_offset: i128) -> bool {
    (l.start as i128 + offset) < (r.end as i128 + second_offset)
        && (r.start as i128 + second_offset) < (l.end as i128 + offset)
}

fn get_repeat_iter(manifest: &Manifest, field: &crate::mir::Field) -> (Vec<i128>, bool) {
    if let Some(repeat) = &field.repeat {
        let stride = repeat.stride;
        match &repeat.source {
            RepeatSource::Count(count) => (
                (0..*count as i128)
                    .map(move |count| count * stride)
                    .collect(),
                true,
            ),
            RepeatSource::Enum(enum_name) => (
                super::search_object(manifest, enum_name)
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
    use crate::mir::{Device, Field, Object, Repeat, Span};

    use super::*;

    #[test]
    fn max_len_exceeded() {
        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".to_owned().with_dummy_span(),
                size_bits: 10.with_dummy_span(),
                fields: vec![Field {
                    name: "my_field".to_owned().with_dummy_span(),
                    field_address: (0..10).with_dummy_span(),
                    ..Default::default()
                }],
                ..Default::default()
            })],
        }
        .into();

        run_pass(&mut start_mir).unwrap();

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".to_owned().with_dummy_span(),
                size_bits: 10.with_dummy_span(),
                fields: vec![Field {
                    name: "my_field".to_owned().with_dummy_span(),
                    field_address: (0..11).with_dummy_span(),
                    ..Default::default()
                }],
                ..Default::default()
            })],
        }
        .into();

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Fieldset `MyReg` has field `my_field` who's address exceeds the given max size bits"
        );

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".to_owned().with_dummy_span(),
                size_bits: 10.with_dummy_span(),
                fields: vec![Field {
                    name: "my_field".to_owned().with_dummy_span(),
                    field_address: (0..10).with_dummy_span(),
                    ..Default::default()
                }],
                ..Default::default()
            })],
        }
        .into();

        run_pass(&mut start_mir).unwrap();

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".to_owned().with_dummy_span(),
                size_bits: 10.with_dummy_span(),
                fields: vec![Field {
                    name: "my_field".to_owned().with_dummy_span(),
                    field_address: (0..11).with_dummy_span(),
                    ..Default::default()
                }],
                ..Default::default()
            })],
        }
        .into();

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Fieldset `MyReg` has field `my_field` who's address exceeds the given max size bits"
        );

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".to_owned().with_dummy_span(),
                size_bits: 10.with_dummy_span(),
                fields: vec![Field {
                    name: "my_field".to_owned().with_dummy_span(),
                    field_address: (0..10).with_dummy_span(),
                    ..Default::default()
                }],
                ..Default::default()
            })],
        }
        .into();

        run_pass(&mut start_mir).unwrap();

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".to_owned().with_dummy_span(),
                size_bits: 10.with_dummy_span(),
                fields: vec![Field {
                    name: "my_field".to_owned().with_dummy_span(),
                    field_address: (0..11).with_dummy_span(),
                    ..Default::default()
                }],
                ..Default::default()
            })],
        }
        .into();

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Fieldset `MyReg` has field `my_field` who's address exceeds the given max size bits"
        );

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".to_owned().with_dummy_span(),
                size_bits: 10.with_dummy_span(),
                fields: vec![Field {
                    name: "my_field".to_owned().with_dummy_span(),
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

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Fieldset `MyReg` has field `my_field` who's address exceeds the given max size bits (at repeat offset: 10)"
        );
    }

    #[test]
    fn overlap() {
        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".to_owned().with_dummy_span(),
                size_bits: 10.with_dummy_span(),
                fields: vec![
                    Field {
                        name: "my_field".to_owned().with_dummy_span(),
                        field_address: (0..5).with_dummy_span(),
                        ..Default::default()
                    },
                    Field {
                        name: "my_field2".to_owned().with_dummy_span(),
                        field_address: (5..10).with_dummy_span(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            })],
        }
        .into();

        run_pass(&mut start_mir).unwrap();

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".to_owned().with_dummy_span(),
                size_bits: 10.with_dummy_span(),
                allow_bit_overlap: true,
                fields: vec![
                    Field {
                        name: "my_field".to_owned().with_dummy_span(),
                        field_address: (0..6).with_dummy_span(),
                        ..Default::default()
                    },
                    Field {
                        name: "my_field2".to_owned().with_dummy_span(),
                        field_address: (5..10).with_dummy_span(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            })],
        }
        .into();

        run_pass(&mut start_mir).unwrap();

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".to_owned().with_dummy_span(),
                size_bits: 10.with_dummy_span(),
                fields: vec![
                    Field {
                        name: "my_field".to_owned().with_dummy_span(),
                        field_address: (0..6).with_dummy_span(),
                        ..Default::default()
                    },
                    Field {
                        name: "my_field2".to_owned().with_dummy_span(),
                        field_address: (5..10).with_dummy_span(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            })],
        }
        .into();

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Fieldset `MyReg` has two overlapping fields: `my_field` and `my_field2`. If this is intended, set the `AllowBitOverlap` option to true"
        );

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".to_owned().with_dummy_span(),
                size_bits: 10.with_dummy_span(),
                fields: vec![
                    Field {
                        name: "my_field".to_owned().with_dummy_span(),
                        field_address: (0..1).with_dummy_span(),
                        repeat: Some(Repeat {
                            source: RepeatSource::Count(6),
                            stride: 1,
                        }),
                        ..Default::default()
                    },
                    Field {
                        name: "my_field2".to_owned().with_dummy_span(),
                        field_address: (5..10).with_dummy_span(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            })],
        }
        .into();

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Fieldset `MyReg` has two overlapping fields: `my_field` (at repeat offset: 5) and `my_field2`. If this is intended, set the `AllowBitOverlap` option to true"
        );
    }
}
