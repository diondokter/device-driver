use std::collections::HashSet;

use crate::mir::{Device, Enum, FieldConversion, Unique};

use super::recurse_objects_mut;

/// Checks if all names are unique to prevent later name collisions.
/// If there is a collision an error is returned.
pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    let mut seen_object_ids = HashSet::new();

    // Nothing must clash with the device name
    seen_object_ids.insert(device.id());

    recurse_objects_mut(&mut device.objects, &mut |object| {
        anyhow::ensure!(
            seen_object_ids.insert(object.id()),
            "Duplicate object name found: `{}`",
            object.name()
        );

        if let Some(field_set) = object.as_field_set() {
            let mut seen_field_names = HashSet::new();
            for field in &field_set.fields {
                anyhow::ensure!(
                    seen_field_names.insert(field.name.clone()),
                    "Duplicate field name found in fieldset `{}`: `{}`",
                    field_set.name,
                    field.name
                );

                if let Some(FieldConversion::Enum {
                    enum_value: enum_value @ Enum { name, variants, .. },
                    ..
                }) = field.field_conversion.as_ref()
                {
                    let mut seen_variant_names = HashSet::new();

                    anyhow::ensure!(
                        seen_object_ids.insert(enum_value.id()),
                        "Duplicate generated enum name `{}` found in fieldset `{}` on field `{}`",
                        name,
                        field_set.name,
                        field.name,
                    );

                    for v in variants.iter() {
                        anyhow::ensure!(
                            seen_variant_names.insert(v.id()),
                            "Duplicate field `{}` found in generated enum `{}` in fieldset `{}` on field `{}`",
                            v.name,
                            name,
                            field_set.name,
                            field.name,
                        );
                    }
                }
            }
        }

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use convert_case::Boundary;

    use crate::mir::{Buffer, EnumVariant, Field, FieldSet, GlobalConfig, Object};

    use super::*;

    #[test]
    #[should_panic(expected = "Duplicate object name found: `MyBuffer`")]
    fn object_names_not_unique() {
        let global_config = GlobalConfig {
            name_word_boundaries: Boundary::defaults_from("-"),
            ..Default::default()
        };

        let mut start_mir = Device {
            name: Some("Device".into()),
            global_config,
            objects: vec![
                Object::Buffer(Buffer {
                    name: "MyBuffer".into(),
                    ..Default::default()
                }),
                Object::Buffer(Buffer {
                    name: "MyBuffer".into(),
                    ..Default::default()
                }),
            ],
        };

        run_pass(&mut start_mir).unwrap();
    }

    #[test]
    #[should_panic(expected = "Duplicate field name found in fieldset `Reg`: `field`")]
    fn field_names_not_unique() {
        let global_config = GlobalConfig {
            name_word_boundaries: Boundary::defaults_from("-"),
            ..Default::default()
        };

        let mut start_mir = Device {
            name: Some("Device".into()),
            global_config,
            objects: vec![Object::FieldSet(FieldSet {
                name: "Reg".into(),
                fields: vec![
                    Field {
                        name: "field".into(),
                        ..Default::default()
                    },
                    Field {
                        name: "field".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Duplicate generated enum name `Enum` found in fieldset `Reg` on field `field2`"
    )]
    fn duplicate_generated_enums() {
        let global_config = GlobalConfig {
            name_word_boundaries: Boundary::defaults_from("-"),
            ..Default::default()
        };

        let mut start_mir = Device {
            name: Some("Device".into()),
            global_config,
            objects: vec![Object::FieldSet(FieldSet {
                name: "Reg".into(),
                fields: vec![
                    Field {
                        name: "field".into(),
                        field_conversion: Some(FieldConversion::Enum {
                            enum_value: Enum {
                                name: "Enum".into(),
                                variants: Default::default(),
                                ..Default::default()
                            },
                            use_try: false,
                        }),
                        ..Default::default()
                    },
                    Field {
                        name: "field2".into(),
                        field_conversion: Some(FieldConversion::Enum {
                            enum_value: Enum {
                                name: "Enum".into(),
                                variants: Default::default(),
                                ..Default::default()
                            },
                            use_try: false,
                        }),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Duplicate field `Variant` found in generated enum `Enum` in fieldset `Reg` on field `field`"
    )]
    fn duplicate_generated_enum_variants() {
        let global_config = GlobalConfig {
            name_word_boundaries: Boundary::defaults_from("-"),
            ..Default::default()
        };

        let mut start_mir = Device {
            name: Some("Device".into()),
            global_config,
            objects: vec![Object::FieldSet(FieldSet {
                name: "Reg".into(),
                fields: vec![Field {
                    name: "field".into(),
                    field_conversion: Some(FieldConversion::Enum {
                        enum_value: Enum {
                            name: "Enum".into(),
                            variants: vec![
                                EnumVariant {
                                    name: "Variant".into(),
                                    ..Default::default()
                                },
                                EnumVariant {
                                    name: "Variant".into(),
                                    ..Default::default()
                                },
                            ],
                            ..Default::default()
                        },
                        use_try: false,
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();
    }
}
