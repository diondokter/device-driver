use std::{borrow::Cow, collections::HashSet};

use device_driver_common::span::SpanExt;

use crate::{
    model::{EnumGenerationStyle, Manifest, Object, Unique, UniqueId},
    passes::Pass,
    search_object,
};
use device_driver_diagnostics::{
    Diagnostics, DynError,
    errors::{
        ConversionTypeTooBig, DifferentBaseTypes, InvalidConversionType,
        InvalidInfallibleConversion, ReferencedObjectDoesNotExist,
    },
};

/// Checks if fields that have conversion and specified no try to be used, are valid in doing so
pub struct FieldConversionValid;

impl Pass for FieldConversionValid {
    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<UniqueId>, DynError> {
        let mut removals = HashSet::new();

        for object in manifest.iter_objects() {
            if let Object::FieldSet(field_set) = object {
                for field in &field_set.fields {
                    if let Some(conversion) = field.field_conversion.as_ref() {
                        let target_object = search_object(manifest, &conversion.type_name);

                        match target_object {
                            Some(Object::Enum(target_enum)) => {
                                let target_enum_size = target_enum.size_bits
                                .ok_or_else(|| DynError::new(
                                    format!(
                                        "target enum `{}` size_bits is none. Should've been set in an earlier pass",
                                        target_enum.name.original()
                                    )
                                ))?;
                                if u64::from(target_enum_size) > field.field_address.len() {
                                    diagnostics.add(ConversionTypeTooBig {
                                        field: field.name.span,
                                        field_address: field.field_address.span,
                                        conversion_type: target_enum.name.span,
                                        conversion: conversion.type_name.span,
                                        field_len: field.field_address.len(),
                                        conversion_len: target_enum_size.into(),
                                    });
                                    removals.insert(field.id_with(field_set.id()));
                                    continue;
                                }

                                if field.base_type != target_enum.base_type {
                                    diagnostics.add(DifferentBaseTypes {
                                        field: field.name.span,
                                        field_base_type: field.base_type.value,
                                        conversion: conversion.type_name.span,
                                        conversion_object: target_enum.name.span,
                                        conversion_base_type: target_enum.base_type.value,
                                    });
                                    removals.insert(field.id_with(field_set.id()));
                                    continue;
                                }

                                if !conversion.fallible {
                                    // Check if we know the value we're converting to and if we can support non-try conversion
                                    match target_enum.generation_style.as_ref().expect(
                                        "Generation style has been set here in an earlier pass",
                                    ) {
                                        EnumGenerationStyle::Fallible => {
                                            diagnostics.add(InvalidInfallibleConversion {
                                                field: field.name.span,
                                                conversion: conversion.type_name.span,
                                                context: vec![
                                                    Cow::from(
                                                        "target only supports fallible conversion",
                                                    )
                                                    .with_span(target_enum.name.span),
                                                ],
                                                existing_type_specifier_content: field
                                                    .get_type_specifier_string(),
                                            });
                                            removals.insert(field.id_with(field_set.id()));
                                            continue;
                                        }
                                        EnumGenerationStyle::InfallibleWithinRange => {
                                            let field_bits = field.field_address.len() as u32;
                                            let enum_bits = target_enum.size_bits.expect(
                                                "Enum size_bits is already set in a previous pass",
                                            );

                                            if field_bits > enum_bits {
                                                diagnostics.add(InvalidInfallibleConversion {
                                    field: field.name.span,
                                                conversion: conversion.type_name.span,
                                                context: vec![
                                                        Cow::from(format!(
                                                                "The field has a size of {field_bits} bits"
                                                            )).with_span(
                                                            field.field_address.span,
                                                        ),
                                                        Cow::from(format!(
                                                                "Target enum only has a size of {enum_bits} bits. This means not all possible field values can be converted to an enum"
                                                            )).with_span(
                                                            target_enum.name.span,
                                                        ),
                                                    ],
                                                existing_type_specifier_content: field.get_type_specifier_string()
                                            });
                                                removals.insert(field.id_with(field_set.id()));
                                                continue;
                                            }
                                        }
                                        EnumGenerationStyle::Fallback => {
                                            // This always works
                                        }
                                    }
                                }
                            }
                            Some(Object::Extern(target_extern)) => {
                                let target_extern_base_type = target_extern.base_type.value.as_fixed_size()
                                .ok_or_else(|| DynError::new(
                                    format!(
                                        "target extern `{}` does not have a fixed size. Should've been checked in an earlier pass",
                                        target_extern.name.original()
                                    )
                                ))?;
                                let target_extern_size = target_extern
                                    .size_bits
                                    .map(|v| v.value)
                                    .unwrap_or(u64::from(target_extern_base_type.size_bits()));

                                if target_extern_size > field.field_address.len() {
                                    diagnostics.add(ConversionTypeTooBig {
                                        field: field.name.span,
                                        field_address: field.field_address.span,
                                        conversion_type: target_extern.name.span,
                                        conversion: conversion.type_name.span,
                                        field_len: field.field_address.len(),
                                        conversion_len: target_extern_size,
                                    });
                                    removals.insert(field.id_with(field_set.id()));
                                    continue;
                                }

                                if field.base_type != target_extern.base_type {
                                    diagnostics.add(DifferentBaseTypes {
                                        field: field.name.span,
                                        field_base_type: field.base_type.value,
                                        conversion: conversion.type_name.span,
                                        conversion_object: target_extern.name.span,
                                        conversion_base_type: target_extern.base_type.value,
                                    });
                                    removals.insert(field.id_with(field_set.id()));
                                    continue;
                                }

                                if !conversion.fallible && !target_extern.supports_infallible {
                                    diagnostics.add(InvalidInfallibleConversion {
                                        field: field.name.span,
                                        conversion: conversion.type_name.span,
                                        context: vec![
                                            Cow::from("target only supports fallible conversion")
                                                .with_span(target_extern.name.span),
                                        ],
                                        existing_type_specifier_content: field
                                            .get_type_specifier_string(),
                                    });
                                    removals.insert(field.id_with(field_set.id()));
                                    continue;
                                }
                            }
                            Some(invalid_object) => {
                                diagnostics.add(InvalidConversionType {
                                    object_reference: conversion.type_name.span,
                                    referenced_object: invalid_object.name_span(),
                                });
                                removals.insert(field.id_with(field_set.id()));
                                continue;
                            }
                            None => {
                                diagnostics.add(ReferencedObjectDoesNotExist {
                                    object_reference: conversion.type_name.span,
                                });
                                removals.insert(field.id_with(field_set.id()));
                                continue;
                            }
                        }
                    }
                }
            }
        }

        Ok(removals)
    }
}
