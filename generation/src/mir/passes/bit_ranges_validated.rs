use std::ops::Range;

use anyhow::ensure;

use crate::mir::{Device, FieldSet, Object};

use super::recurse_objects_mut;

/// Validate that the bit ranges of fields fall within the max size and don't have overlap if they're not allowed
pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    recurse_objects_mut(&mut device.objects, &mut |object| match object {
        Object::Register(r) => validate_field_set(&r.field_set, &r.name),
        Object::Command(c) => {
            if let Some(field_set_in) = &c.field_set_in {
                validate_field_set(field_set_in, &format!("{} (in)", c.name))?;
            }
            if let Some(field_set_out) = &c.field_set_out {
                validate_field_set(field_set_out, &format!("{} (out)", c.name))?;
            }

            Ok(())
        }
        Object::Block(_) | Object::Buffer(_) | Object::Ref(_) => Ok(()),
    })
}

fn validate_field_set(field_set: &FieldSet, object_name: &str) -> anyhow::Result<()> {
    validate_len(field_set, object_name)?;
    if !field_set.allow_bit_overlap {
        validate_overlap(field_set, object_name)?;
    }

    Ok(())
}

fn validate_len(field_set: &FieldSet, object_name: &str) -> anyhow::Result<()> {
    for field in &field_set.fields {
        ensure!(
            field.field_address.end <= field_set.size_bits,
            "Object `{object_name}` has field `{}` who's address exceeds the given max size bits",
            field.name
        );

        ensure!(
            field.field_address.clone().count() > 0,
            "Object `{object_name}` has field `{}` that is 0 bits. This is likely a mistake",
            field.name
        );
    }

    Ok(())
}

fn validate_overlap(field_set: &FieldSet, object_name: &str) -> anyhow::Result<()> {
    for (i, field) in field_set.fields.iter().enumerate() {
        for second_field in &field_set.fields[(i + 1).min(field_set.fields.len())..] {
            ensure!(
                !ranges_overlap(&field.field_address, &second_field.field_address),
                "Object `{object_name}` has two overlapping fields: `{}` and `{}`. If this is intended, set the `AllowBitOverlap` option to true",
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
    use crate::mir::{Command, Field, Register};

    use super::*;

    #[test]
    fn max_len_exceeded() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Register(Register {
                name: "MyReg".into(),
                field_set: FieldSet {
                    size_bits: 10,
                    fields: vec![Field {
                        name: "my_field".into(),
                        field_address: 0..10,
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Register(Register {
                name: "MyReg".into(),
                field_set: FieldSet {
                    size_bits: 10,
                    fields: vec![Field {
                        name: "my_field".into(),
                        field_address: 0..11,
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                ..Default::default()
            })],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Object `MyReg` has field `my_field` who's address exceeds the given max size bits"
        );

        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Command(Command {
                name: "MyReg".into(),
                field_set_in: Some(FieldSet {
                    size_bits: 10,
                    fields: vec![Field {
                        name: "my_field".into(),
                        field_address: 0..10,
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Command(Command {
                name: "MyReg".into(),
                field_set_in: Some(FieldSet {
                    size_bits: 10,
                    fields: vec![Field {
                        name: "my_field".into(),
                        field_address: 0..11,
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            })],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Object `MyReg (in)` has field `my_field` who's address exceeds the given max size bits"
        );

        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Command(Command {
                name: "MyReg".into(),
                field_set_out: Some(FieldSet {
                    size_bits: 10,
                    fields: vec![Field {
                        name: "my_field".into(),
                        field_address: 0..10,
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Command(Command {
                name: "MyReg".into(),
                field_set_out: Some(FieldSet {
                    size_bits: 10,
                    fields: vec![Field {
                        name: "my_field".into(),
                        field_address: 0..11,
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            })],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Object `MyReg (out)` has field `my_field` who's address exceeds the given max size bits"
        );
    }

    #[test]
    fn overlap_register() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Register(Register {
                name: "MyReg".into(),
                field_set: FieldSet {
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
                },
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Register(Register {
                name: "MyReg".into(),
                field_set: FieldSet {
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
                },
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Register(Register {
                name: "MyReg".into(),
                field_set: FieldSet {
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
                },
                ..Default::default()
            })],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Object `MyReg` has two overlapping fields: `my_field` and `my_field2`. If this is intended, set the `AllowBitOverlap` option to true"
        );
    }

    #[test]
    fn overlap_command_in() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Command(Command {
                name: "MyReg".into(),
                field_set_in: Some(FieldSet {
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
                }),
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Command(Command {
                name: "MyReg".into(),
                field_set_in: Some(FieldSet {
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
                }),
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Command(Command {
                name: "MyReg".into(),
                field_set_in: Some(FieldSet {
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
                }),
                ..Default::default()
            })],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Object `MyReg (in)` has two overlapping fields: `my_field` and `my_field2`. If this is intended, set the `AllowBitOverlap` option to true"
        );
    }

    #[test]
    fn overlap_command_out() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Command(Command {
                name: "MyReg".into(),
                field_set_out: Some(FieldSet {
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
                }),
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Command(Command {
                name: "MyReg".into(),
                field_set_out: Some(FieldSet {
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
                }),
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Command(Command {
                name: "MyReg".into(),
                field_set_out: Some(FieldSet {
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
                }),
                ..Default::default()
            })],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Object `MyReg (out)` has two overlapping fields: `my_field` and `my_field2`. If this is intended, set the `AllowBitOverlap` option to true"
        );
    }
}
