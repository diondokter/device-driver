use std::collections::HashSet;

use crate::mir::{Device, Enum, FieldConversion, Unique};

use super::recurse_objects_mut;

/// Checks if all names are unique to prevent later name collisions.
/// If there is a collision an error is returned.
pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    let mut seen_object_ids = HashSet::new();
    let mut generated_type_ids = HashSet::new();

    recurse_objects_mut(&mut device.objects, &mut |object| {
        anyhow::ensure!(
            seen_object_ids.insert(object.id()),
            "Duplicate object name found: \"{}\"",
            object.name()
        );

        for field_set in object.field_sets() {
            let mut seen_field_names = HashSet::new();
            for field in field_set {
                anyhow::ensure!(
                    seen_field_names.insert(field.name.clone()),
                    "Duplicate field name found in object \"{}\": \"{}\"",
                    object.name(),
                    field.name
                );

                if let Some(FieldConversion::Enum {
                    enum_value: enum_value @ Enum { name, variants, .. },
                    ..
                }) = field.field_conversion.as_ref()
                {
                    let mut seen_variant_names = HashSet::new();

                    anyhow::ensure!(
                        generated_type_ids.insert(enum_value.id()),
                        "Duplicate generated enum name \"{}\" found in object \"{}\" on field \"{}\"",
                        name,
                        object.name(),
                        field.name,
                    );

                    for v in variants.iter() {
                        anyhow::ensure!(
                            seen_variant_names.insert(v.id()),
                            "Duplicate field \"{}\" found in generated enum \"{}\" in object \"{}\" on field \"{}\"",
                            v.name,
                            name,
                            object.name(),
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

    use crate::mir::{Buffer, Cfg, EnumVariant, Field, GlobalConfig, Object, Register};

    use super::*;

    #[test]
    #[should_panic(expected = "Duplicate object name found: \"MyBuffer\"")]
    fn object_names_not_unique() {
        let global_config = GlobalConfig {
            name_word_boundaries: Boundary::list_from("-"),
            ..Default::default()
        };

        let mut start_mir = Device {
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
    #[should_panic(expected = "Duplicate field name found in object \"Reg\": \"field\"")]
    fn field_names_not_unique() {
        let global_config = GlobalConfig {
            name_word_boundaries: Boundary::list_from("-"),
            ..Default::default()
        };

        let mut start_mir = Device {
            global_config,
            objects: vec![Object::Register(Register {
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
        expected = "Duplicate generated enum name \"Enum\" found in object \"Reg\" on field \"field2\""
    )]
    fn duplicate_generated_enums() {
        let global_config = GlobalConfig {
            name_word_boundaries: Boundary::list_from("-"),
            ..Default::default()
        };

        let mut start_mir = Device {
            global_config,
            objects: vec![Object::Register(Register {
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
        expected = "Duplicate field \"Variant\" found in generated enum \"Enum\" in object \"Reg\" on field \"field\""
    )]
    fn duplicate_generated_enum_variants() {
        let global_config = GlobalConfig {
            name_word_boundaries: Boundary::list_from("-"),
            ..Default::default()
        };

        let mut start_mir = Device {
            global_config,
            objects: vec![Object::Register(Register {
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

    #[test]
    fn duplicate_cfg_generated_enum_variants() {
        let global_config = GlobalConfig {
            name_word_boundaries: Boundary::list_from("-"),
            ..Default::default()
        };

        let mut start_mir = Device {
            global_config,
            objects: vec![Object::Register(Register {
                name: "Reg".into(),
                fields: vec![Field {
                    name: "field".into(),
                    field_conversion: Some(FieldConversion::Enum {
                        enum_value: Enum {
                            name: "Enum".into(),
                            variants: vec![
                                EnumVariant {
                                    name: "Variant".into(),
                                    cfg_attr: Cfg::new(Some("windows")),
                                    ..Default::default()
                                },
                                EnumVariant {
                                    name: "Variant".into(),
                                    cfg_attr: Cfg::new(Some("unix")),
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
