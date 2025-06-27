use anyhow::{bail, ensure};
use itertools::Itertools;

use crate::mir::{Device, EnumGenerationStyle, EnumValue, FieldConversion, Unique};

use super::recurse_objects_mut;

/// Checks if enums are fully specified and determines the generation style
pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    recurse_objects_mut(&mut device.objects, &mut |object| {
        let object_name = object.name().to_string();

        for field in object.field_sets_mut().flat_map(|fs| fs.fields.iter_mut()) {
            if let Some(FieldConversion::Enum {
                enum_value: ec,
                use_try,
            }) = field.field_conversion.as_mut()
            {
                let field_bits = field.field_address.clone().count();
                let highest_value = (1 << field_bits) - 1;

                ensure!(
                    field_bits <= 128,
                    "Enum `{}` is too big to fit in 128-bit in object `{}` on field `{}`",
                    &ec.name,
                    object_name,
                    &field.name
                );

                ensure!(
                    !ec.variants.is_empty(),
                    "Enum `{}` has no variants which is not allowed. Add at least one variant",
                    &ec.name,
                );

                // Record all variant values
                let mut seen_values = Vec::new();
                for variant in ec.variants.iter_mut() {
                    match &mut variant.value {
                        val @ EnumValue::Unspecified => {
                            let assigned_value =
                                seen_values.last().map(|(val, _)| *val + 1).unwrap_or(0);
                            *val = EnumValue::Specified(assigned_value);
                            seen_values.push((assigned_value, variant.id()));
                        }
                        EnumValue::Specified(num) => {
                            seen_values.push((*num, variant.id()));
                        }
                        EnumValue::Default | EnumValue::CatchAll => {
                            let assigned_value =
                                seen_values.last().map(|(val, _)| *val + 1).unwrap_or(0);
                            seen_values.push((assigned_value, variant.id()));
                        }
                    }
                }

                let duplicates = seen_values
                    .iter()
                    .duplicates()
                    .map(|(num, name)| format!("{name}: {num}"))
                    .collect::<Vec<_>>();

                ensure!(
                    duplicates.is_empty(),
                    "Duplicated assigned value(s) for enum `{}` in object `{}` on field `{}`: {duplicates:?}",
                    &ec.name,
                    object_name,
                    &field.name
                );

                // Check if all bits are covered or if there's a fallback variant
                let has_fallback = ec
                    .variants
                    .iter()
                    .any(|v| matches!(v.value, EnumValue::Default | EnumValue::CatchAll));
                let has_bits_covered = (0..=highest_value)
                    .all(|val| seen_values.iter().any(|(seen_val, _)| val == *seen_val));

                ec.generation_style = Some(if has_fallback || has_bits_covered {
                    EnumGenerationStyle::Infallible {
                        bit_size: field_bits as u32,
                    }
                } else {
                    EnumGenerationStyle::Fallible
                });

                // Check if the enum has variants that fall outside of the available bits
                if let Some(too_big_variant) =
                    seen_values.iter().find(|(val, _)| *val > highest_value)
                {
                    bail!(
                        "The value of variant `{}` is too high for enum `{}` in object `{}` on field `{}`: {} (max = {highest_value})",
                        too_big_variant.1,
                        &ec.name,
                        object_name,
                        &field.name,
                        too_big_variant.0
                    )
                }

                // Check whether the enum has more than one default
                ensure!(
                    ec.variants.iter().filter(|v| v.value.is_default()).count() < 2,
                    "More than one default defined on enum `{}` in object `{}` on field `{}`",
                    &ec.name,
                    object_name,
                    &field.name
                );

                // Check whether the enum has more than one catch all
                ensure!(
                    ec.variants
                        .iter()
                        .filter(|v| v.value.is_catch_all())
                        .count()
                        < 2,
                    "More than one catch all defined on enum `{}` in object `{}` on field `{}`",
                    &ec.name,
                    object_name,
                    &field.name
                );

                if ec.generation_style.as_ref().unwrap().is_fallible() && !*use_try {
                    bail!(
                        "Not all bitpatterns are covered on non-try conversion enum `{}` in object `{}` on field `{}`",
                        &ec.name,
                        object_name,
                        &field.name
                    );
                }
            }
        }

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use crate::mir::{Command, Enum, EnumVariant, Field, FieldSet, Object};

    use super::*;

    #[test]
    fn enum_values_correct() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Command(Command {
                name: "MyCommand".into(),
                field_set_out: Some(FieldSet {
                    fields: vec![Field {
                        field_conversion: Some(FieldConversion::Enum {
                            enum_value: Enum::new(
                                Default::default(),
                                "MyEnum".into(),
                                vec![
                                    EnumVariant {
                                        name: "var0".into(),
                                        value: EnumValue::Specified(1),
                                        ..Default::default()
                                    },
                                    EnumVariant {
                                        name: "var1".into(),
                                        value: EnumValue::Unspecified,
                                        ..Default::default()
                                    },
                                    EnumVariant {
                                        name: "var2".into(),
                                        value: EnumValue::Unspecified,
                                        ..Default::default()
                                    },
                                    EnumVariant {
                                        name: "var3".into(),
                                        value: EnumValue::Specified(0),
                                        ..Default::default()
                                    },
                                ],
                            ),
                            use_try: false,
                        }),
                        field_address: 0..2,
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            })],
        };

        let end_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Command(Command {
                name: "MyCommand".into(),
                field_set_out: Some(FieldSet {
                    fields: vec![Field {
                        field_conversion: Some(FieldConversion::Enum {
                            enum_value: Enum::new_with_style(
                                Default::default(),
                                "MyEnum".into(),
                                vec![
                                    EnumVariant {
                                        name: "var0".into(),
                                        value: EnumValue::Specified(1),
                                        ..Default::default()
                                    },
                                    EnumVariant {
                                        name: "var1".into(),
                                        value: EnumValue::Specified(2),
                                        ..Default::default()
                                    },
                                    EnumVariant {
                                        name: "var2".into(),
                                        value: EnumValue::Specified(3),
                                        ..Default::default()
                                    },
                                    EnumVariant {
                                        name: "var3".into(),
                                        value: EnumValue::Specified(0),
                                        ..Default::default()
                                    },
                                ],
                                EnumGenerationStyle::Infallible { bit_size: 2 },
                            ),
                            use_try: false,
                        }),
                        field_address: 0..2,
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        assert_eq!(start_mir, end_mir);
    }

    #[test]
    fn enum_values_infallible_with_fallback() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Command(Command {
                name: "MyCommand".into(),
                field_set_out: Some(FieldSet {
                    fields: vec![Field {
                        field_conversion: Some(FieldConversion::Enum {
                            enum_value: Enum::new(
                                Default::default(),
                                "MyEnum".into(),
                                vec![
                                    EnumVariant {
                                        name: "var0".into(),
                                        value: EnumValue::Unspecified,
                                        ..Default::default()
                                    },
                                    EnumVariant {
                                        name: "var1".into(),
                                        value: EnumValue::Default,
                                        ..Default::default()
                                    },
                                ],
                            ),
                            use_try: false,
                        }),
                        field_address: 0..2,
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            })],
        };

        let end_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Command(Command {
                name: "MyCommand".into(),
                field_set_out: Some(FieldSet {
                    fields: vec![Field {
                        field_conversion: Some(FieldConversion::Enum {
                            enum_value: Enum::new_with_style(
                                Default::default(),
                                "MyEnum".into(),
                                vec![
                                    EnumVariant {
                                        name: "var0".into(),
                                        value: EnumValue::Specified(0),
                                        ..Default::default()
                                    },
                                    EnumVariant {
                                        name: "var1".into(),
                                        value: EnumValue::Default,
                                        ..Default::default()
                                    },
                                ],
                                EnumGenerationStyle::Infallible { bit_size: 2 },
                            ),
                            use_try: false,
                        }),
                        field_address: 0..2,
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        assert_eq!(start_mir, end_mir);
    }

    #[test]
    fn enum_values_fallible() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Command(Command {
                name: "MyCommand".into(),
                field_set_out: Some(FieldSet {
                    fields: vec![Field {
                        field_conversion: Some(FieldConversion::Enum {
                            enum_value: Enum::new(
                                Default::default(),
                                "MyEnum".into(),
                                vec![EnumVariant {
                                    name: "var0".into(),
                                    value: EnumValue::Unspecified,
                                    ..Default::default()
                                }],
                            ),
                            use_try: true,
                        }),
                        field_address: 0..2,
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            })],
        };

        let end_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Command(Command {
                name: "MyCommand".into(),
                field_set_out: Some(FieldSet {
                    fields: vec![Field {
                        field_conversion: Some(FieldConversion::Enum {
                            enum_value: Enum::new_with_style(
                                Default::default(),
                                "MyEnum".into(),
                                vec![EnumVariant {
                                    name: "var0".into(),
                                    value: EnumValue::Specified(0),
                                    ..Default::default()
                                }],
                                EnumGenerationStyle::Fallible,
                            ),
                            use_try: true,
                        }),
                        field_address: 0..2,
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        assert_eq!(start_mir, end_mir);
    }

    #[test]
    fn enum_values_dont_fit() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Command(Command {
                name: "MyCommand".into(),
                field_set_out: Some(FieldSet {
                    fields: vec![Field {
                        name: "MyField".into(),
                        field_conversion: Some(FieldConversion::Enum {
                            enum_value: Enum::new(
                                Default::default(),
                                "MyEnum".into(),
                                vec![
                                    EnumVariant {
                                        name: "var0".into(),
                                        value: EnumValue::Unspecified,
                                        ..Default::default()
                                    },
                                    EnumVariant {
                                        name: "var0".into(),
                                        value: EnumValue::Unspecified,
                                        ..Default::default()
                                    },
                                    EnumVariant {
                                        name: "var0".into(),
                                        value: EnumValue::Unspecified,
                                        ..Default::default()
                                    },
                                ],
                            ),
                            use_try: false,
                        }),
                        field_address: 0..1,
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            })],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "The value of variant `var0` is too high for enum `MyEnum` in object `MyCommand` on field `MyField`: 2 (max = 1)"
        );
    }

    #[test]
    fn enum_values_no_duplicates() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Command(Command {
                name: "MyCommand".into(),
                field_set_out: Some(FieldSet {
                    fields: vec![Field {
                        name: "MyField".into(),
                        field_conversion: Some(FieldConversion::Enum {
                            enum_value: Enum::new(
                                Default::default(),
                                "MyEnum".into(),
                                vec![
                                    EnumVariant {
                                        name: "var0".into(),
                                        value: EnumValue::Unspecified,
                                        ..Default::default()
                                    },
                                    EnumVariant {
                                        name: "var0".into(),
                                        value: EnumValue::Specified(0),
                                        ..Default::default()
                                    },
                                ],
                            ),
                            use_try: false,
                        }),
                        field_address: 0..1,
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            })],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Duplicated assigned value(s) for enum `MyEnum` in object `MyCommand` on field `MyField`: [\"var0: 0\"]"
        );
    }
}
