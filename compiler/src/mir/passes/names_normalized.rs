use convert_case::Case;

use crate::mir::{LendingIterator, Manifest, Object, Repeat, RepeatSource};

/// Changes all names of all objects, enums, enum variants and fieldsets to either Pascal case or snake case
///
/// - PascalCase: Object names, enum names, enum variant names
/// - snake_case: Field names
pub fn run_pass(manifest: &mut Manifest) -> miette::Result<()> {
    let mut iter = manifest.iter_objects_with_config_mut();
    while let Some((object, config)) = iter.next() {
        if let Object::Device(_) = object {
            // The name rules for devices are slightly different and are done in a different pass
            continue;
        }

        let boundaries = config
            .name_word_boundaries
            .as_deref()
            .unwrap_or(&const { convert_case::Boundary::defaults() });

        let pascal_converter = convert_case::Converter::new()
            .set_boundaries(boundaries)
            .to_case(Case::Pascal);
        let snake_converter = convert_case::Converter::new()
            .set_boundaries(boundaries)
            .to_case(Case::Snake);

        *object.name_mut() = pascal_converter.convert(object.name_mut());

        for fs_name in object.field_set_refs_mut() {
            fs_name.0 = pascal_converter.convert(fs_name.0.clone());
        }

        if let Object::FieldSet(field_set) = object {
            for field in field_set.fields.iter_mut() {
                field.name = snake_converter.convert(&field.name);

                if let Some(conversion) = field.field_conversion.as_mut() {
                    conversion.type_name = pascal_converter.convert(&conversion.type_name);
                }

                if let Some(Repeat {
                    source: RepeatSource::Enum(name),
                    ..
                }) = field.repeat.as_mut()
                {
                    *name = pascal_converter.convert(&name);
                }
            }
        }

        if let Object::Enum(enum_value) = object {
            for variant in enum_value.variants.iter_mut() {
                variant.name = pascal_converter.convert(&variant.name);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use convert_case::Boundary;

    use crate::mir::{
        Buffer, Device, DeviceConfig, Enum, EnumVariant, Field, FieldConversion, FieldSet, Object,
        Register,
    };

    use super::*;

    #[test]
    fn names_normalized() {
        let global_config = DeviceConfig {
            name_word_boundaries: Some(Boundary::defaults_from("-")),
            ..Default::default()
        };

        let mut start_mir: Manifest = Device {
            name: "Device".into(),
            device_config: global_config.clone(),
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
                            field_conversion: Some(FieldConversion {
                                type_name: "mY-enum".into(),
                                use_try: false,
                            }),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }),
                Object::Enum(Enum {
                    name: "mY-enum".into(),
                    variants: vec![EnumVariant {
                        name: "eNum-Variant".into(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
            ],
        }
        .into();

        let end_mir: Manifest = Device {
            name: "Device".into(),
            device_config: global_config,
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
                            field_conversion: Some(FieldConversion {
                                type_name: "MyEnum".into(),
                                use_try: false,
                            }),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }),
                Object::Enum(Enum {
                    name: "MyEnum".into(),
                    variants: vec![EnumVariant {
                        name: "EnumVariant".into(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
            ],
        }
        .into();

        run_pass(&mut start_mir).unwrap();

        pretty_assertions::assert_eq!(start_mir, end_mir);
    }
}
