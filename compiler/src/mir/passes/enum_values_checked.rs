use itertools::Itertools;
use miette::{bail, ensure};

use crate::mir::{BaseType, Device, EnumGenerationStyle, EnumValue, Integer, Object, Unique};

use super::recurse_objects_mut;

/// Checks if enums are fully specified and determines the generation style
pub fn run_pass(device: &mut Device) -> miette::Result<()> {
    recurse_objects_mut(&mut device.objects, &mut |object| {
        let object_name = object.name().to_string();

        if let Object::Enum(enum_value) = object {
            ensure!(
                !enum_value.variants.is_empty(),
                "Enum `{object_name}` has no variants which is not allowed. Add at least one variant",
            );

            // Record all variant values
            let seen_values = enum_value
                .iter_variants_with_discriminant_mut()
                .map(|(discriminant, variant)| {
                    if variant.value.is_unspecified() {
                        variant.value = EnumValue::Specified(discriminant);
                    }
                    (discriminant, variant.id())
                })
                .collect_vec();

            let duplicates = seen_values
                .iter()
                .duplicates()
                .map(|(num, name)| format!("{name}: {num}"))
                .collect::<Vec<_>>();

            ensure!(
                duplicates.is_empty(),
                "Duplicated assigned value(s) for enum `{object_name}`: {duplicates:?}",
            );

            let seen_min = seen_values
                .iter()
                .map(|(val, _)| *val)
                .min()
                .unwrap_or_default();
            let seen_max = seen_values
                .iter()
                .map(|(val, _)| *val)
                .max()
                .unwrap_or_default();

            let base_type_integer = match enum_value.base_type {
                BaseType::Unspecified => Integer::find_smallest(
                    seen_min,
                    seen_max,
                    enum_value.size_bits.unwrap_or_default(),
                ),
                BaseType::Bool => {
                    bail!("Enum `{object_name}` uses a bool as base type, which is not allowed")
                }
                BaseType::Uint => {
                    let integer = Integer::find_smallest(
                        seen_min,
                        seen_max,
                        enum_value.size_bits.unwrap_or_default(),
                    );
                    ensure!(
                        integer.map(|i| !i.is_signed()).unwrap_or(true),
                        "Enum `{object_name}` has a variant that uses a negative number, but the base type was specified as unsigned"
                    );

                    integer
                }
                BaseType::Int => Integer::find_smallest(
                    seen_min.min(-1),
                    seen_max,
                    enum_value.size_bits.unwrap_or_default(),
                ),
                BaseType::FixedSize(integer) => {
                    ensure!(
                        integer.size_bits() >= enum_value.size_bits.unwrap_or_default(),
                        "Enum `{object_name}` has specified a 'size-bits' that is larger than its base type. This is not allowed"
                    );
                    Some(integer)
                }
            };

            let Some(base_type_integer) = base_type_integer else {
                bail!(
                    "No valid base type could be selected for enum `{object_name}`. Either the specified size-bits or the variants cannot fit within any of the supported integer types"
                )
            };

            let size_bits = match enum_value.size_bits {
                None => {
                    if base_type_integer.is_signed() {
                        u32::max(
                            i128::BITS - seen_min.unsigned_abs().leading_zeros(),
                            i128::BITS - seen_max.unsigned_abs().leading_zeros(),
                        ) + seen_min.is_negative() as u32
                    } else {
                        i128::BITS - seen_max.leading_zeros()
                    }
                }
                Some(size_bits) => size_bits,
            };

            enum_value.base_type = BaseType::FixedSize(base_type_integer);
            enum_value.size_bits = Some(size_bits);

            let highest_value = (1 << size_bits) - 1;

            // Check if all bits are covered or if there's a fallback variant
            let has_fallback = enum_value
                .variants
                .iter()
                .any(|v| matches!(v.value, EnumValue::Default | EnumValue::CatchAll));
            let has_bits_covered = (0..=highest_value)
                .all(|val| seen_values.iter().any(|(seen_val, _)| val == *seen_val));

            enum_value.generation_style = Some(match (has_fallback, has_bits_covered) {
                (true, _) => EnumGenerationStyle::Fallback,
                (false, true) => EnumGenerationStyle::InfallibleWithinRange,
                (false, false) => EnumGenerationStyle::Fallible,
            });

            // Check if the enum has variants that fall outside of the available bits
            if let Some(too_big_variant) = seen_values.iter().find(|(val, _)| *val > highest_value)
            {
                bail!(
                    "The value of variant `{}` is too high for enum `{object_name}`: {} (max = {highest_value})",
                    too_big_variant.1,
                    too_big_variant.0
                )
            }

            // Check whether the enum has more than one default
            ensure!(
                enum_value
                    .variants
                    .iter()
                    .filter(|v| v.value.is_default())
                    .count()
                    < 2,
                "More than one default defined on enum `{object_name}`",
            );

            // Check whether the enum has more than one catch all
            ensure!(
                enum_value
                    .variants
                    .iter()
                    .filter(|v| v.value.is_catch_all())
                    .count()
                    < 2,
                "More than one catch all defined on enum `{object_name}`",
            );
        }

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use crate::mir::{Enum, EnumVariant, Object};

    use super::*;

    #[test]
    fn enum_values_correct() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Enum(Enum::new(
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
                BaseType::Unspecified,
                Some(2),
            ))],
        };

        let end_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Enum(Enum::new_with_style(
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
                BaseType::FixedSize(crate::mir::Integer::U8),
                Some(2),
                EnumGenerationStyle::InfallibleWithinRange,
            ))],
        };

        run_pass(&mut start_mir).unwrap();

        assert_eq!(start_mir, end_mir);
    }

    #[test]
    fn enum_values_infallible_with_fallback() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Enum(Enum::new(
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
                BaseType::Unspecified,
                Some(8),
            ))],
        };

        let end_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Enum(Enum::new_with_style(
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
                BaseType::FixedSize(crate::mir::Integer::U8),
                Some(8),
                EnumGenerationStyle::Fallback,
            ))],
        };

        run_pass(&mut start_mir).unwrap();

        assert_eq!(start_mir, end_mir);
    }

    #[test]
    fn enum_values_fallible() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Enum(Enum::new(
                Default::default(),
                "MyEnum".into(),
                vec![EnumVariant {
                    name: "var0".into(),
                    value: EnumValue::Unspecified,
                    ..Default::default()
                }],
                BaseType::Unspecified,
                Some(16),
            ))],
        };

        let end_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Enum(Enum::new_with_style(
                Default::default(),
                "MyEnum".into(),
                vec![EnumVariant {
                    name: "var0".into(),
                    value: EnumValue::Specified(0),
                    ..Default::default()
                }],
                BaseType::FixedSize(Integer::U16),
                Some(16),
                EnumGenerationStyle::Fallible,
            ))],
        };

        run_pass(&mut start_mir).unwrap();

        assert_eq!(start_mir, end_mir);
    }

    #[test]
    fn enum_values_dont_fit() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Enum(Enum::new(
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
                BaseType::Unspecified,
                Some(1),
            ))],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "The value of variant `var0` is too high for enum `MyEnum`: 2 (max = 1)"
        );
    }

    #[test]
    fn enum_values_no_duplicates() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Enum(Enum::new(
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
                BaseType::Unspecified,
                Some(8),
            ))],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Duplicated assigned value(s) for enum `MyEnum`: [\"var0: 0\"]"
        );
    }
}
