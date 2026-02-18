use std::collections::{HashMap, HashSet};

use device_driver_common::{
    span::SpanExt,
    specifiers::{BaseType, Integer},
};
use itertools::Itertools;

use crate::model::{
    EnumGenerationStyle, EnumValue, LendingIterator, Manifest, Object, Unique, UniqueId,
};
use device_driver_diagnostics::{
    Diagnostics,
    errors::{
        DuplicateVariantValue, EmptyEnum, EnumBadBasetype, EnumMultipleCatchalls,
        EnumMultipleDefaults, EnumNoAutoBaseTypeSelected, EnumSizeBitsBiggerThanBaseType,
        VariantValuesTooHigh, VariantValuesTooLow,
    },
};

/// Checks if enums are fully specified and determines the generation style
pub fn run_pass(manifest: &mut Manifest, diagnostics: &mut Diagnostics) -> HashSet<UniqueId> {
    let mut removals = HashSet::new();

    let mut iter = manifest.iter_objects_with_config_mut();
    while let Some((object, _)) = iter.next() {
        let Object::Enum(enum_value) = object else {
            continue;
        };

        if enum_value.variants.is_empty() {
            diagnostics.add(EmptyEnum {
                enum_node: enum_value.span,
            });
            removals.insert(enum_value.id());
            continue;
        }

        // Record all variant values
        let e_id = enum_value.id();
        let seen_values = enum_value
            .iter_variants_with_discriminant_mut()
            .map(|(discriminant, variant)| {
                if variant.value.is_unspecified() {
                    variant.value = EnumValue::Specified(discriminant);
                }
                (discriminant, variant.id_with(e_id.clone()))
            })
            .collect_vec();

        let mut seen_values_map = HashMap::<i128, Vec<&UniqueId>>::new();
        for (variant_value, variant_id) in &seen_values {
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
                    info: "all enums must have an integer as base type",
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

                if integer.is_some_and(|i| i.is_signed()) {
                    // TODO: Make separate error type (same as below)
                    diagnostics.add(EnumBadBasetype {
                        enum_name: enum_value.name.span,
                        base_type: enum_value.base_type.span,
                        info: "enums must use a signed integer when any variant has a negative value",
                        context: vec![format!("variant with negative value: {seen_min}").with_span(seen_min_id.span())],
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
                        enum_size_bits: enum_value.size_bits.unwrap_or_default(),
                        base_type_size_bits: integer.size_bits(),
                    });
                    removals.insert(enum_value.id());
                    continue;
                }

                if !integer.is_signed() && seen_min.is_negative() {
                    // TODO: Make separate error type (same as above)
                    diagnostics.add(EnumBadBasetype {
                        enum_name: enum_value.name.span,
                        base_type: enum_value.base_type.span,
                        info: "enums must use a signed integer when any variant has a negative value",
                        context: vec![format!("variant with negative value: {seen_min}").with_span(seen_min_id.span())],
                    });
                    removals.insert(enum_value.id());
                    continue;
                }

                Some(integer)
            }
        };

        let Some(base_type_integer) = base_type_integer else {
            diagnostics.add(EnumNoAutoBaseTypeSelected {
                enum_name: enum_value.name.span,
            });
            removals.insert(enum_value.id());
            continue;
        };

        let size_bits = match enum_value.size_bits {
            None => base_type_integer
                .bits_required(*seen_min, *seen_max)
                .min(base_type_integer.size_bits()),
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
        let too_high_values = seen_values
            .iter()
            .filter(|(val, _)| val > all_values.end())
            .map(|(_, id)| id.span())
            .collect::<Vec<_>>();
        let too_low_values = seen_values
            .iter()
            .filter(|(val, _)| val < all_values.start())
            .map(|(_, id)| id.span())
            .collect::<Vec<_>>();

        if !too_high_values.is_empty() {
            diagnostics.add(VariantValuesTooHigh {
                variant_names: too_high_values,
                enum_name: enum_value.name.span,
                max_value: *all_values.end(),
                size_bits,
            });
            removals.insert(enum_value.id());
            continue;
        }
        if !too_low_values.is_empty() {
            diagnostics.add(VariantValuesTooLow {
                variant_names: too_low_values,
                enum_name: enum_value.name.span,
                min_value: *all_values.start(),
                size_bits,
            });
            removals.insert(enum_value.id());
            continue;
        }

        // Check whether the enum has more than one default
        let default_variants = enum_value
            .variants
            .iter()
            .filter(|v| v.value.is_default())
            .map(|v| v.name.span)
            .collect::<Vec<_>>();

        if default_variants.len() > 1 {
            diagnostics.add(EnumMultipleDefaults {
                variant_names: default_variants,
                enum_name: enum_value.name.span,
            });

            // Set all but the first default to unspecified. This aids keeping the generated code available
            let mut defaults_seen = 0;
            for variant in &mut enum_value.variants {
                if variant.value.is_default() {
                    if defaults_seen != 0 {
                        variant.value = EnumValue::Unspecified;
                    }
                    defaults_seen += 1;
                }
            }
        }

        // Check whether the enum has more than one catch all
        let catch_all_variants = enum_value
            .variants
            .iter()
            .filter(|v| v.value.is_catch_all())
            .map(|v| v.name.span)
            .collect::<Vec<_>>();

        if catch_all_variants.len() > 1 {
            diagnostics.add(EnumMultipleCatchalls {
                variant_names: catch_all_variants,
                enum_name: enum_value.name.span,
            });

            // Set all but the first catch-all to unspecified. This aids keeping the generated code available
            let mut catch_alls_seen = 0;
            for variant in &mut enum_value.variants {
                if variant.value.is_catch_all() {
                    if catch_alls_seen != 0 {
                        variant.value = EnumValue::Unspecified;
                    }
                    catch_alls_seen += 1;
                }
            }
        }
    }

    removals
}

#[cfg(test)]
mod tests {
    use device_driver_common::span::{Span, SpanExt};

    use crate::model::{Device, Enum, EnumVariant, Object};

    use super::*;

    #[test]
    fn enum_values_correct() {
        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::Enum(Enum::new(
                Default::default(),
                "MyEnum".into_with_dummy_span(),
                vec![
                    EnumVariant {
                        name: "var0".into_with_dummy_span(),
                        value: EnumValue::Specified(1),
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "var1".into_with_dummy_span(),
                        value: EnumValue::Unspecified,
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "var2".into_with_dummy_span(),
                        value: EnumValue::Unspecified,
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "var3".into_with_dummy_span(),
                        value: EnumValue::Specified(0),
                        ..Default::default()
                    },
                ],
                BaseType::Unspecified.with_dummy_span(),
                Some(2),
                Span::default(),
            ))],
            span: Span::default(),
        }
        .into();

        let end_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::Enum(Enum::new_with_style(
                Default::default(),
                "MyEnum".into_with_dummy_span(),
                vec![
                    EnumVariant {
                        name: "var0".into_with_dummy_span(),
                        value: EnumValue::Specified(1),
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "var1".into_with_dummy_span(),
                        value: EnumValue::Specified(2),
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "var2".into_with_dummy_span(),
                        value: EnumValue::Specified(3),
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "var3".into_with_dummy_span(),
                        value: EnumValue::Specified(0),
                        ..Default::default()
                    },
                ],
                BaseType::FixedSize(Integer::U8).with_dummy_span(),
                Some(2),
                EnumGenerationStyle::InfallibleWithinRange,
                Span::default(),
            ))],
            span: Span::default(),
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);

        assert!(!diagnostics.has_error());
        assert_eq!(start_mir, end_mir);
    }

    #[test]
    fn enum_values_infallible_with_fallback() {
        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::Enum(Enum::new(
                Default::default(),
                "MyEnum".into_with_dummy_span(),
                vec![
                    EnumVariant {
                        name: "var0".into_with_dummy_span(),
                        value: EnumValue::Unspecified,
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "var1".into_with_dummy_span(),
                        value: EnumValue::Default,
                        ..Default::default()
                    },
                ],
                BaseType::Unspecified.with_dummy_span(),
                Some(8),
                Span::default(),
            ))],
            span: Span::default(),
        }
        .into();

        let end_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::Enum(Enum::new_with_style(
                Default::default(),
                "MyEnum".into_with_dummy_span(),
                vec![
                    EnumVariant {
                        name: "var0".into_with_dummy_span(),
                        value: EnumValue::Specified(0),
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "var1".into_with_dummy_span(),
                        value: EnumValue::Default,
                        ..Default::default()
                    },
                ],
                BaseType::FixedSize(Integer::U8).with_dummy_span(),
                Some(8),
                EnumGenerationStyle::Fallback,
                Span::default(),
            ))],
            span: Span::default(),
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);

        assert!(!diagnostics.has_error());
        assert_eq!(start_mir, end_mir);
    }

    #[test]
    fn enum_values_fallible() {
        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::Enum(Enum::new(
                Default::default(),
                "MyEnum".into_with_dummy_span(),
                vec![EnumVariant {
                    name: "var0".into_with_dummy_span(),
                    value: EnumValue::Unspecified,
                    ..Default::default()
                }],
                BaseType::Unspecified.with_dummy_span(),
                Some(16),
                Span::default(),
            ))],
            span: Span::default(),
        }
        .into();

        let end_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::Enum(Enum::new_with_style(
                Default::default(),
                "MyEnum".into_with_dummy_span(),
                vec![EnumVariant {
                    name: "var0".into_with_dummy_span(),
                    value: EnumValue::Specified(0),
                    ..Default::default()
                }],
                BaseType::FixedSize(Integer::U16).with_dummy_span(),
                Some(16),
                EnumGenerationStyle::Fallible,
                Span::default(),
            ))],
            span: Span::default(),
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);

        assert!(!diagnostics.has_error());
        assert_eq!(start_mir, end_mir);
    }

    #[test]
    fn enum_values_dont_fit() {
        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::Enum(Enum::new(
                Default::default(),
                "MyEnum".into_with_dummy_span(),
                vec![
                    EnumVariant {
                        name: "var0".into_with_dummy_span(),
                        value: EnumValue::Unspecified,
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "var0".into_with_dummy_span(),
                        value: EnumValue::Unspecified,
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "var0".into_with_dummy_span(),
                        value: EnumValue::Unspecified,
                        ..Default::default()
                    },
                ],
                BaseType::Unspecified.with_dummy_span(),
                Some(1),
                Span::default(),
            ))],
            span: Span::default(),
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        let removals = run_pass(&mut start_mir, &mut diagnostics);

        assert!(diagnostics.has_error());
        assert!(removals.contains(&UniqueId::new_test("MyEnum".into())));
    }

    #[test]
    fn enum_values_no_duplicates() {
        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::Enum(Enum::new(
                Default::default(),
                "MyEnum".into_with_dummy_span(),
                vec![
                    EnumVariant {
                        name: "var0".into_with_dummy_span(),
                        value: EnumValue::Unspecified,
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "var0".into_with_dummy_span(),
                        value: EnumValue::Specified(0),
                        ..Default::default()
                    },
                ],
                BaseType::Unspecified.with_dummy_span(),
                Some(8),
                Span::default(),
            ))],
            span: Span::default(),
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        let removals = run_pass(&mut start_mir, &mut diagnostics);

        assert!(diagnostics.has_error());
        assert_eq!(removals.len(), 1);
        assert!(removals.contains(&UniqueId::new_test("MyEnum".into())));
    }
}
