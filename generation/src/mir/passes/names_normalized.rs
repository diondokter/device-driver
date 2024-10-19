use convert_case::Case;

use crate::mir::{Device, Enum, FieldConversion};

use super::recurse_objects_mut;

/// Changes all names of all objects, enums and enum variants to either Pascal case or snake case
///
/// - PascalCase: Object names, enum names, enum variant names
/// - snake_case: Field names
pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    let boundaries = device.global_config.name_word_boundaries.clone();

    let pascal_converter = convert_case::Converter::new()
        .set_boundaries(&boundaries)
        .to_case(Case::Pascal);
    let snake_converter = convert_case::Converter::new()
        .set_boundaries(&boundaries)
        .to_case(Case::Snake);

    recurse_objects_mut(&mut device.objects, &mut |object| {
        *object.name_mut() = pascal_converter.convert(object.name_mut());

        for field in object.field_sets_mut().flatten() {
            field.name = snake_converter.convert(&field.name);
            if let Some(FieldConversion::Enum {
                enum_value: Enum { name, variants, .. },
                ..
            }) = field.field_conversion.as_mut()
            {
                *name = pascal_converter.convert(&*name);

                for v in variants.iter_mut() {
                    v.name = pascal_converter.convert(&v.name)
                }
            }
        }

        if let Some(ref_object) = object.as_ref_object_mut() {
            *ref_object.object_override.name_mut() =
                pascal_converter.convert(ref_object.object_override.name_mut());
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
    fn names_normalized() {
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
            global_config: global_config.clone(),
            objects: vec![
                Object::Register(Register {
                    name: "my-reGister".into(),
                    fields: vec![
                        Field {
                            name: "my-fielD".into(),
                            ..Default::default()
                        },
                        Field {
                            name: "my-fielD2".into(),
                            field_conversion: Some(FieldConversion::Enum {
                                enum_value: Enum {
                                    name: "mY-enum".into(),
                                    variants: vec![EnumVariant {
                                        name: "eNum-Variant".into(),
                                        ..Default::default()
                                    }],
                                    ..Default::default()
                                },
                                use_try: false,
                            }),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }),
                Object::Buffer(Buffer {
                    name: "my-buffer".into(),
                    ..Default::default()
                }),
            ],
        };

        let end_mir = Device {
            global_config,
            objects: vec![
                Object::Register(Register {
                    name: "MyRegister".into(),
                    fields: vec![
                        Field {
                            name: "my_field".into(),
                            ..Default::default()
                        },
                        Field {
                            name: "my_field2".into(),
                            field_conversion: Some(FieldConversion::Enum {
                                enum_value: Enum {
                                    name: "MyEnum".into(),
                                    variants: vec![EnumVariant {
                                        name: "EnumVariant".into(),
                                        ..Default::default()
                                    }],
                                    ..Default::default()
                                },
                                use_try: false,
                            }),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }),
                Object::Buffer(Buffer {
                    name: "MyBuffer".into(),
                    ..Default::default()
                }),
            ],
        };

        run_pass(&mut start_mir).unwrap();

        assert_eq!(start_mir, end_mir);
    }
}
