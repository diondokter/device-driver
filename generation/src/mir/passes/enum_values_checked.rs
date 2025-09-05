use anyhow::{bail, ensure};
use itertools::Itertools;

use crate::mir::{BaseType, Device, EnumGenerationStyle, EnumValue, Integer, Object, Unique};

use super::recurse_objects_mut;

/// Checks if enums are fully specified and determines the generation style
pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    recurse_objects_mut(&mut device.objects, &mut |object| {
        let object_name = object.name().to_string();

        if let Object::Enum(enum_value) = object {
            let size_bits = match enum_value.size_bits {
                None => {
                    let size_bits = match enum_value.base_type {
                        BaseType::Unspecified | BaseType::Uint | BaseType::Int => bail!(
                            "Enum `{}` has an unknown size-bits. Either specify a fixed integer or bool as the base type or specify the size-bits directly",
                            object_name
                        ),
                        BaseType::Bool => 1,
                        BaseType::FixedSize(integer) => integer.size_bits(),
                    };
                    enum_value.size_bits = Some(size_bits);
                    size_bits
                }
                Some(size_bits) => size_bits,
            };

            let highest_value = (1 << size_bits) - 1;

            ensure!(
                size_bits <= 128,
                "Enum `{object_name}` is too big to fit in 128-bit",
            );

            ensure!(
                !enum_value.variants.is_empty(),
                "Enum `{object_name}` has no variants which is not allowed. Add at least one variant",
            );

            // Record all variant values
            let mut seen_values = Vec::new();
            for variant in enum_value.variants.iter_mut() {
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
                "Duplicated assigned value(s) for enum `{object_name}`: {duplicates:?}",
            );

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

            // Specify the enum basetype into a fixed size integer
            enum_value.base_type = match enum_value.base_type {
                BaseType::Bool => {
                    bail!("Enum `{object_name}` has a bool base type. Only integers are supported")
                }
                BaseType::Uint | BaseType::Unspecified => {
                    match find_smallest_integer(seen_min, seen_max, size_bits) {
                        Some(integer) if integer.is_signed() => bail!(
                            "Enum `{object_name}` has a uint base type, but also has a negative variant"
                        ),
                        Some(integer) => BaseType::FixedSize(integer),
                        None => bail!(
                            "Enum `{object_name}` has a variant that doesn't fit in any of the supported unsigned integer types. Min: {seen_min}, max: {seen_max}"
                        ),
                    }
                }
                BaseType::Int => match find_smallest_integer(seen_min.min(-1), seen_max, size_bits)
                {
                    Some(integer) => BaseType::FixedSize(integer),
                    None => bail!(
                        "Enum `{object_name}` has a variant that doesn't fit in any of the supported signed integer types. Min: {seen_min}, max: {seen_max}"
                    ),
                },
                BaseType::FixedSize(integer) => BaseType::FixedSize(integer),
            };
        }

        Ok(())
    })
}

fn find_smallest_integer(min: i128, max: i128, size_bits: u32) -> Option<Integer> {
    Some(match (min, max, size_bits) {
        (0.., ..0x1_00, ..=8) => Integer::U8,
        (0.., ..0x1_0000, ..=16) => Integer::U16,
        (0.., ..0x1_0000_0000, ..=32) => Integer::U32,
        (0.., ..0x1_0000_0000_0000_0000, ..=64) => Integer::U64,
        (-0x80.., ..0x80, ..=8) => Integer::I8,
        (-0x8000.., ..0x8000, ..=16) => Integer::I16,
        (-0x8000_00000.., ..0x8000_0000, ..=32) => Integer::I32,
        (-0x8000_0000_0000_00000.., ..0x8000_0000_0000_0000, ..=32) => Integer::I64,
        _ => return None,
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
