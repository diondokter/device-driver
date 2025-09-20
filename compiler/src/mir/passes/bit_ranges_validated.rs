use std::ops::Range;

use miette::ensure;

use crate::mir::{Device, FieldSet};

use super::recurse_objects_mut;

/// Validate that the bit ranges of fields fall within the max size and don't have overlap if they're not allowed
pub fn run_pass(device: &mut Device) -> miette::Result<()> {
    recurse_objects_mut(&mut device.objects, &mut |object| {
        if let Some(field_set) = object.as_field_set() {
            validate_len(field_set)?;
            if !field_set.allow_bit_overlap {
                validate_overlap(field_set)?;
            }
        }

        Ok(())
    })
}

fn validate_len(field_set: &FieldSet) -> miette::Result<()> {
    for field in &field_set.fields {
        ensure!(
            field.field_address.end <= field_set.size_bits,
            "Fieldset `{}` has field `{}` who's address exceeds the given max size bits",
            field_set.name,
            field.name
        );

        ensure!(
            field.field_address.clone().count() > 0,
            "Fieldset `{}` has field `{}` that is 0 bits. This is likely a mistake",
            field_set.name,
            field.name
        );
    }

    Ok(())
}

fn validate_overlap(field_set: &FieldSet) -> miette::Result<()> {
    for (i, field) in field_set.fields.iter().enumerate() {
        for second_field in &field_set.fields[(i + 1).min(field_set.fields.len())..] {
            ensure!(
                !ranges_overlap(&field.field_address, &second_field.field_address),
                "Fieldset `{}` has two overlapping fields: `{}` and `{}`. If this is intended, set the `AllowBitOverlap` option to true",
                field_set.name,
                field.name,
                second_field.name
            )
        }
    }

    Ok(())
}

fn ranges_overlap(l: &Range<u32>, r: &Range<u32>) -> bool {
    l.start < r.end && r.start < l.end
}

#[cfg(test)]
mod tests {
    use crate::mir::{Field, Object};

    use super::*;

    #[test]
    fn max_len_exceeded() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
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
            global_config: Default::default(),
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
            global_config: Default::default(),
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
            global_config: Default::default(),
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
            global_config: Default::default(),
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
            global_config: Default::default(),
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
    }

    #[test]
    fn overlap() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
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
            global_config: Default::default(),
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
            global_config: Default::default(),
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
    }
}
