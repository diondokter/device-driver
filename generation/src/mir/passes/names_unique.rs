use std::collections::HashSet;

use crate::mir::{Device, FieldConversion};

use super::recurse_objects;

pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    let mut seen_object_names = HashSet::new();
    let mut generated_type_names = HashSet::new();

    recurse_objects(&mut device.objects, &mut |object| {
        anyhow::ensure!(
            seen_object_names.insert(object.name().clone()),
            "Duplicate object name found: \"{}\"",
            object.name()
        );

        let mut seen_field_names = HashSet::new();
        for field in object
            .fields()
            .unwrap_or_else(|| Box::new(std::iter::empty()))
        {
            anyhow::ensure!(
                seen_field_names.insert(field.name.clone()),
                "Duplicate field name found in object \"{}\": \"{}\"",
                object.name(),
                field.name
            );

            if let Some(FieldConversion::Enum { name, variants }) = field.field_conversion.as_ref()
            {
                let mut seen_variant_names = HashSet::new();

                anyhow::ensure!(
                    generated_type_names.insert(name.clone()),
                    "Duplicate generated enum name \"{}\" found in object \"{}\" on field \"{}\"",
                    name,
                    object.name(),
                    field.name,
                );

                for v in variants.iter() {
                    anyhow::ensure!(
                        seen_variant_names.insert(v.name.clone()),
                        "Duplicate field \"{}\" found in generated enum \"{}\" in object \"{}\" on field \"{}\"",
                        v.name,
                        name,
                        object.name(),
                        field.name,
                    );
                }
            }
        }

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use convert_case::Boundary;

    use crate::mir::{Buffer, EnumVariant, Field, GlobalConfig, Object, Register};

    use super::*;

    #[test]
    #[should_panic(expected = "Duplicate object name found: \"MyBuffer\"")]
    fn object_names_not_unique() {
        let global_config = GlobalConfig {
            default_register_access: Default::default(),
            default_field_access: Default::default(),
            default_buffer_access: Default::default(),
            default_byte_order: Default::default(),
            default_bit_order: Default::default(),
            register_address_type: Default::default(),
            command_address_type: Default::default(),
            buffer_address_type: Default::default(),
            name_word_boundaries: Boundary::list_from("-"),
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
            default_register_access: Default::default(),
            default_field_access: Default::default(),
            default_buffer_access: Default::default(),
            default_byte_order: Default::default(),
            default_bit_order: Default::default(),
            register_address_type: Default::default(),
            command_address_type: Default::default(),
            buffer_address_type: Default::default(),
            name_word_boundaries: Boundary::list_from("-"),
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
            default_register_access: Default::default(),
            default_field_access: Default::default(),
            default_buffer_access: Default::default(),
            default_byte_order: Default::default(),
            default_bit_order: Default::default(),
            register_address_type: Default::default(),
            command_address_type: Default::default(),
            buffer_address_type: Default::default(),
            name_word_boundaries: Boundary::list_from("-"),
        };

        let mut start_mir = Device {
            global_config,
            objects: vec![Object::Register(Register {
                name: "Reg".into(),
                fields: vec![
                    Field {
                        name: "field".into(),
                        field_conversion: Some(FieldConversion::Enum {
                            name: "Enum".into(),
                            variants: Default::default(),
                        }),
                        ..Default::default()
                    },
                    Field {
                        name: "field2".into(),
                        field_conversion: Some(FieldConversion::Enum {
                            name: "Enum".into(),
                            variants: Default::default(),
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
            default_register_access: Default::default(),
            default_field_access: Default::default(),
            default_buffer_access: Default::default(),
            default_byte_order: Default::default(),
            default_bit_order: Default::default(),
            register_address_type: Default::default(),
            command_address_type: Default::default(),
            buffer_address_type: Default::default(),
            name_word_boundaries: Boundary::list_from("-"),
        };

        let mut start_mir = Device {
            global_config,
            objects: vec![Object::Register(Register {
                name: "Reg".into(),
                fields: vec![Field {
                    name: "field".into(),
                    field_conversion: Some(FieldConversion::Enum {
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
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();
    }
}
