use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use miette::{LabeledSpan, bail, ensure};

use crate::{
    mir::{
        BaseType, EnumGenerationStyle, EnumValue, Integer, LendingIterator, Manifest, Object,
        Unique, UniqueId,
    },
    reporting::{
        Diagnostics,
        errors::{
            DuplicateVariantValue, EmptyEnum, EnumBadBasetype, EnumSizeBitsBiggerThanBaseType,
        },
    },
};

/// Checks if enums are fully specified and determines the generation style
pub fn run_pass(
    manifest: &mut Manifest,
    diagnostics: &mut Diagnostics,
) -> miette::Result<HashSet<UniqueId>> {
    let mut removals = HashSet::new();

    let mut iter = manifest.iter_objects_with_config_mut();
    while let Some((object, _)) = iter.next() {
        let object_name = object.name().to_string();

        let Object::Enum(enum_value) = object else {
            continue;
        };

        if enum_value.variants.is_empty() {
            diagnostics.add(EmptyEnum {
                enum_name: enum_value.name.span,
            });
            removals.insert(enum_value.id());
            continue;
        }

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

        let mut seen_values_map = HashMap::<i128, Vec<&UniqueId>>::new();
        for (variant_value, variant_id) in seen_values.iter() {
            seen_values_map
                .entry(*variant_value)
                .or_default()
                .push(variant_id);
        }
        for (value, variants) in seen_values_map
            .into_iter()
            .sorted_unstable_by_key(|(value, _)| *value)
        {
            if variants.len() > 1 {
                diagnostics.add(DuplicateVariantValue {
                    duplicates: variants.iter().map(|id| id.span()).collect(),
                    value,
                });
                removals.insert(enum_value.id());
            }
        }

        let (seen_min, seen_min_id) = seen_values
            .iter()
            .min_by_key(|(val, _)| val)
            .expect("Enums must not be empty");
        let (seen_max, _) = seen_values
            .iter()
            .max_by_key(|(val, _)| val)
            .expect("Enums must not be empty");

        let base_type_integer = match enum_value.base_type.value {
            BaseType::Unspecified => Integer::find_smallest(
                *seen_min,
                *seen_max,
                enum_value.size_bits.unwrap_or_default(),
            ),
            BaseType::Bool => {
                diagnostics.add(EnumBadBasetype {
                    enum_name: enum_value.name.span,
                    base_type: enum_value.base_type.span,
                    help: "All enums must have an integer as base type",
                    context: vec![],
                });
                removals.insert(enum_value.id());
                continue;
            }
            BaseType::Uint => {
                let integer = Integer::find_smallest(
                    *seen_min,
                    *seen_max,
                    enum_value.size_bits.unwrap_or_default(),
                );

                if integer.map(|i| i.is_signed()).unwrap_or(false) {
                    diagnostics.add(EnumBadBasetype {
                        enum_name: enum_value.name.span,
                        base_type: enum_value.base_type.span,
                        help: "All enums must use a signed integer if it contains a variant with a negative value",
                        context: vec![LabeledSpan::new_with_span(Some(format!("Variant with negative value: {seen_min}")), seen_min_id.span())],
                    });
                    removals.insert(enum_value.id());
                    continue;
                }

                integer
            }
            BaseType::Int => Integer::find_smallest(
                (*seen_min).min(-1),
                *seen_max,
                enum_value.size_bits.unwrap_or_default(),
            ),
            BaseType::FixedSize(integer) => {
                if enum_value.size_bits.unwrap_or_default() > integer.size_bits() {
                    diagnostics.add(EnumSizeBitsBiggerThanBaseType {
                        enum_name: enum_value.name.span,
                        base_type: enum_value.base_type.span,
                        size_bits: enum_value.size_bits.unwrap_or_default(),
                    });
                    removals.insert(enum_value.id());
                    continue;
                }

                if !integer.is_signed() && seen_min.is_negative() {
                    diagnostics.add(EnumBadBasetype {
                        enum_name: enum_value.name.span,
                        base_type: enum_value.base_type.span,
                        help: "All enums must use a signed integer if it contains a variant with a negative value",
                        context: vec![LabeledSpan::new_with_span(Some(format!("Variant with negative value: {seen_min}")), seen_min_id.span())],
                    });
                    removals.insert(enum_value.id());
                    continue;
                }

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

        enum_value.base_type.value = BaseType::FixedSize(base_type_integer);
        enum_value.size_bits = Some(size_bits);

        let all_values = if base_type_integer.is_signed() {
            let max = (1 << size_bits) / 2;
            -max..=max - 1
        } else {
            0..=(1 << size_bits) - 1
        };

        // Check if all bits are covered or if there's a fallback variant
        let has_fallback = enum_value
            .variants
            .iter()
            .any(|v| matches!(v.value, EnumValue::Default | EnumValue::CatchAll));
        let has_bits_covered = all_values
            .clone()
            .all(|val| seen_values.iter().any(|(seen_val, _)| val == *seen_val));

        enum_value.generation_style = Some(match (has_fallback, has_bits_covered) {
            (true, _) => EnumGenerationStyle::Fallback,
            (false, true) => EnumGenerationStyle::InfallibleWithinRange,
            (false, false) => EnumGenerationStyle::Fallible,
        });

        // Check if the enum has variants that fall outside of the available bits
        if let Some(too_big_variant) = seen_values.iter().find(|(val, _)| val > all_values.end()) {
            bail!(
                "The value of variant `{}` is too high for enum `{object_name}`: {} (max = {})",
                too_big_variant.1,
                too_big_variant.0,
                all_values.end()
            )
        }
        if let Some(too_small_variant) =
            seen_values.iter().find(|(val, _)| val < all_values.start())
        {
            bail!(
                "The value of variant `{}` is too low for enum `{object_name}`: {} (min = {})",
                too_small_variant.1,
                too_small_variant.0,
                all_values.start()
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

    Ok(removals)
}

#[cfg(test)]
mod tests {
    use crate::mir::{Device, Enum, EnumVariant, Object, Span};

    use super::*;

    #[test]
    fn enum_values_correct() {
        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::Enum(Enum::new(
                Default::default(),
                "MyEnum".to_owned().with_dummy_span(),
                vec![
                    EnumVariant {
                        name: "var0".to_owned().with_dummy_span(),
                        value: EnumValue::Specified(1),
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "var1".to_owned().with_dummy_span(),
                        value: EnumValue::Unspecified,
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "var2".to_owned().with_dummy_span(),
                        value: EnumValue::Unspecified,
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "var3".to_owned().with_dummy_span(),
                        value: EnumValue::Specified(0),
                        ..Default::default()
                    },
                ],
                BaseType::Unspecified.with_dummy_span(),
                Some(2),
            ))],
        }
        .into();

        let end_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::Enum(Enum::new_with_style(
                Default::default(),
                "MyEnum".to_owned().with_dummy_span(),
                vec![
                    EnumVariant {
                        name: "var0".to_owned().with_dummy_span(),
                        value: EnumValue::Specified(1),
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "var1".to_owned().with_dummy_span(),
                        value: EnumValue::Specified(2),
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "var2".to_owned().with_dummy_span(),
                        value: EnumValue::Specified(3),
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "var3".to_owned().with_dummy_span(),
                        value: EnumValue::Specified(0),
                        ..Default::default()
                    },
                ],
                BaseType::FixedSize(crate::mir::Integer::U8).with_dummy_span(),
                Some(2),
                EnumGenerationStyle::InfallibleWithinRange,
            ))],
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics).unwrap();

        assert!(!diagnostics.has_error());
        assert_eq!(start_mir, end_mir);
    }

    #[test]
    fn enum_values_infallible_with_fallback() {
        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::Enum(Enum::new(
                Default::default(),
                "MyEnum".to_owned().with_dummy_span(),
                vec![
                    EnumVariant {
                        name: "var0".to_owned().with_dummy_span(),
                        value: EnumValue::Unspecified,
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "var1".to_owned().with_dummy_span(),
                        value: EnumValue::Default,
                        ..Default::default()
                    },
                ],
                BaseType::Unspecified.with_dummy_span(),
                Some(8),
            ))],
        }
        .into();

        let end_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::Enum(Enum::new_with_style(
                Default::default(),
                "MyEnum".to_owned().with_dummy_span(),
                vec![
                    EnumVariant {
                        name: "var0".to_owned().with_dummy_span(),
                        value: EnumValue::Specified(0),
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "var1".to_owned().with_dummy_span(),
                        value: EnumValue::Default,
                        ..Default::default()
                    },
                ],
                BaseType::FixedSize(crate::mir::Integer::U8).with_dummy_span(),
                Some(8),
                EnumGenerationStyle::Fallback,
            ))],
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics).unwrap();

        assert!(!diagnostics.has_error());
        assert_eq!(start_mir, end_mir);
    }

    #[test]
    fn enum_values_fallible() {
        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::Enum(Enum::new(
                Default::default(),
                "MyEnum".to_owned().with_dummy_span(),
                vec![EnumVariant {
                    name: "var0".to_owned().with_dummy_span(),
                    value: EnumValue::Unspecified,
                    ..Default::default()
                }],
                BaseType::Unspecified.with_dummy_span(),
                Some(16),
            ))],
        }
        .into();

        let end_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::Enum(Enum::new_with_style(
                Default::default(),
                "MyEnum".to_owned().with_dummy_span(),
                vec![EnumVariant {
                    name: "var0".to_owned().with_dummy_span(),
                    value: EnumValue::Specified(0),
                    ..Default::default()
                }],
                BaseType::FixedSize(Integer::U16).with_dummy_span(),
                Some(16),
                EnumGenerationStyle::Fallible,
            ))],
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics).unwrap();

        assert!(!diagnostics.has_error());
        assert_eq!(start_mir, end_mir);
    }

    #[test]
    fn enum_values_dont_fit() {
        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::Enum(Enum::new(
                Default::default(),
                "MyEnum".to_owned().with_dummy_span(),
                vec![
                    EnumVariant {
                        name: "var0".to_owned().with_dummy_span(),
                        value: EnumValue::Unspecified,
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "var0".to_owned().with_dummy_span(),
                        value: EnumValue::Unspecified,
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "var0".to_owned().with_dummy_span(),
                        value: EnumValue::Unspecified,
                        ..Default::default()
                    },
                ],
                BaseType::Unspecified.with_dummy_span(),
                Some(1),
            ))],
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        assert_eq!(
            run_pass(&mut start_mir, &mut diagnostics)
                .unwrap_err()
                .to_string(),
            "The value of variant `var0` is too high for enum `MyEnum`: 2 (max = 1)"
        );

        // TODO: Remove UI assert and assert that the enum is in the removal list
    }

    #[test]
    fn enum_values_no_duplicates() {
        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::Enum(Enum::new(
                Default::default(),
                "MyEnum".to_owned().with_dummy_span(),
                vec![
                    EnumVariant {
                        name: "var0".to_owned().with_dummy_span(),
                        value: EnumValue::Unspecified,
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "var0".to_owned().with_dummy_span(),
                        value: EnumValue::Specified(0),
                        ..Default::default()
                    },
                ],
                BaseType::Unspecified.with_dummy_span(),
                Some(8),
            ))],
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        let removals = run_pass(&mut start_mir, &mut diagnostics).unwrap();

        assert!(diagnostics.has_error());
        assert_eq!(removals.len(), 1);
        assert!(removals.contains(&UniqueId::new_test("MyEnum".to_owned())));
    }
}
