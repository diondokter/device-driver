use convert_case::Case;

use crate::mir::{Device, Enum, FieldConversion};

use super::recurse_objects_mut;

/// Changes all names of all objects, enums, enum variants and fieldsets to either Pascal case or snake case
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

        for fs_name in object.field_set_refs_mut() {
            fs_name.0 = pascal_converter.convert(fs_name.0.clone());
        }

        if let Some(field_set) = object.as_field_set_mut() {
            for field in field_set.fields.iter_mut() {
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
        }

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use convert_case::Boundary;

    use crate::mir::{Buffer, EnumVariant, Field, FieldSet, GlobalConfig, Object, Register};

    use super::*;

    #[test]
    fn names_normalized() {
        let global_config = GlobalConfig {
            name_word_boundaries: Boundary::list_from("-"),
            ..Default::default()
        };

        let mut start_mir = Device {
            name: None,
            global_config: global_config.clone(),
            objects: vec![
                Object::Register(Register {
                    name: "my-reGister".into(),
                    field_set_ref: crate::mir::FieldSetRef("my-fieldseT".into()),
                    ..Default::default()
                }),
                Object::Buffer(Buffer {
                    name: "my-buffer".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "my-fieldseT".into(),
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
            ],
        };

        let end_mir = Device {
            name: None,
            global_config,
            objects: vec![
                Object::Register(Register {
                    name: "MyRegister".into(),
                    field_set_ref: crate::mir::FieldSetRef("MyFieldset".into()),
                    ..Default::default()
                }),
                Object::Buffer(Buffer {
                    name: "MyBuffer".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "MyFieldset".into(),
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
            ],
        };

        run_pass(&mut start_mir).unwrap();

        pretty_assertions::assert_eq!(start_mir, end_mir);
    }
}
