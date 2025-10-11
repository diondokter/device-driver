use std::ops::Range;

use miette::ensure;

use crate::mir::{Device, FieldSet, RepeatSource, passes::recurse_objects};

/// Validate that the bit ranges of fields fall within the max size and don't have overlap if they're not allowed
pub fn run_pass(device: &mut Device) -> miette::Result<()> {
    recurse_objects(&device.objects, &mut |object| {
        if let Some(field_set) = object.as_field_set() {
            validate_len(field_set, device)?;
            if !field_set.allow_bit_overlap {
                validate_overlap(field_set, device)?;
            }
        }

        Ok(())
    })
}

fn validate_len(field_set: &FieldSet, device: &Device) -> miette::Result<()> {
    for field in &field_set.fields {
        ensure!(
            field.field_address.clone().count() > 0,
            "Fieldset `{}` has field `{}` that is 0 bits. This is likely a mistake",
            field_set.name,
            field.name
        );

        let (offset_iter, repeated) = get_repeat_iter(device, field);

        for offset in offset_iter {
            let repeat_info = if repeated {
                format!(" (at repeat offset: {offset})")
            } else {
                "".into()
            };

            ensure!(
                (field.field_address.end as i128 + offset) <= field_set.size_bits as i128,
                "Fieldset `{}` has field `{}` who's address exceeds the given max size bits{repeat_info}",
                field_set.name,
                field.name
            );
            ensure!(
                (field.field_address.start as i128 + offset) >= 0,
                "Fieldset `{}` has field `{}` who's address is negative{repeat_info}",
                field_set.name,
                field.name
            );
        }
    }

    Ok(())
}

fn validate_overlap(field_set: &FieldSet, device: &Device) -> miette::Result<()> {
    for (i, field) in field_set.fields.iter().enumerate() {
        let (offsets, repeated) = get_repeat_iter(device, field);

        for (j, second_field) in field_set.fields.iter().enumerate() {
            let (second_offsets, second_repeated) = get_repeat_iter(device, second_field);

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

fn get_repeat_iter(device: &Device, field: &crate::mir::Field) -> (Vec<i128>, bool) {
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
                super::search_object(&device.objects, enum_name)
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
    use crate::mir::{Field, Object, Repeat};

    use super::*;

    #[test]
    fn max_len_exceeded() {
        let mut start_mir = Device {
            name: None,
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".into(),
                size_bits: 10,
                fields: vec![Field {
                    name: "my_field".into(),
                    field_address: 0..10,
                    ..Default::default()
                }],
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        let mut start_mir = Device {
            name: None,
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".into(),
                size_bits: 10,
                fields: vec![Field {
                    name: "my_field".into(),
                    field_address: 0..11,
                    ..Default::default()
                }],
                ..Default::default()
            })],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Fieldset `MyReg` has field `my_field` who's address exceeds the given max size bits"
        );

        let mut start_mir = Device {
            name: None,
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".into(),
                size_bits: 10,
                fields: vec![Field {
                    name: "my_field".into(),
                    field_address: 0..10,
                    ..Default::default()
                }],
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        let mut start_mir = Device {
            name: None,
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".into(),
                size_bits: 10,
                fields: vec![Field {
                    name: "my_field".into(),
                    field_address: 0..11,
                    ..Default::default()
                }],
                ..Default::default()
            })],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Fieldset `MyReg` has field `my_field` who's address exceeds the given max size bits"
        );

        let mut start_mir = Device {
            name: None,
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".into(),
                size_bits: 10,
                fields: vec![Field {
                    name: "my_field".into(),
                    field_address: 0..10,
                    ..Default::default()
                }],
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        let mut start_mir = Device {
            name: None,
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".into(),
                size_bits: 10,
                fields: vec![Field {
                    name: "my_field".into(),
                    field_address: 0..11,
                    ..Default::default()
                }],
                ..Default::default()
            })],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Fieldset `MyReg` has field `my_field` who's address exceeds the given max size bits"
        );

        let mut start_mir = Device {
            name: None,
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".into(),
                size_bits: 10,
                fields: vec![Field {
                    name: "my_field".into(),
                    field_address: 0..5,
                    repeat: Some(Repeat {
                        source: RepeatSource::Count(3),
                        stride: 5,
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            })],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Fieldset `MyReg` has field `my_field` who's address exceeds the given max size bits (at repeat offset: 10)"
        );
    }

    #[test]
    fn overlap() {
        let mut start_mir = Device {
            name: None,
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".into(),
                size_bits: 10,
                fields: vec![
                    Field {
                        name: "my_field".into(),
                        field_address: 0..5,
                        ..Default::default()
                    },
                    Field {
                        name: "my_field2".into(),
                        field_address: 5..10,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        let mut start_mir = Device {
            name: None,
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".into(),
                size_bits: 10,
                allow_bit_overlap: true,
                fields: vec![
                    Field {
                        name: "my_field".into(),
                        field_address: 0..6,
                        ..Default::default()
                    },
                    Field {
                        name: "my_field2".into(),
                        field_address: 5..10,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        let mut start_mir = Device {
            name: None,
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".into(),
                size_bits: 10,
                fields: vec![
                    Field {
                        name: "my_field".into(),
                        field_address: 0..6,
                        ..Default::default()
                    },
                    Field {
                        name: "my_field2".into(),
                        field_address: 5..10,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            })],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Fieldset `MyReg` has two overlapping fields: `my_field` and `my_field2`. If this is intended, set the `AllowBitOverlap` option to true"
        );

        let mut start_mir = Device {
            name: None,
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyReg".into(),
                size_bits: 10,
                fields: vec![
                    Field {
                        name: "my_field".into(),
                        field_address: 0..1,
                        repeat: Some(Repeat {
                            source: RepeatSource::Count(6),
                            stride: 1,
                        }),
                        ..Default::default()
                    },
                    Field {
                        name: "my_field2".into(),
                        field_address: 5..10,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            })],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Fieldset `MyReg` has two overlapping fields: `my_field` (at repeat offset: 5) and `my_field2`. If this is intended, set the `AllowBitOverlap` option to true"
        );
    }
}
