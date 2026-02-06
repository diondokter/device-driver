use std::collections::HashSet;

use miette::LabeledSpan;

use crate::{
    model::{EnumGenerationStyle, Manifest, Object, Unique, UniqueId},
    search_object,
};
use device_driver_diagnostics::{
    Diagnostics,
    errors::{
        DifferentBaseTypes, InvalidInfallibleConversion, ReferencedObjectDoesNotExist,
        ReferencedObjectInvalid,
    },
};

/// Checks if fields that have conversion and specified no try to be used, are valid in doing so
pub fn run_pass(manifest: &mut Manifest, diagnostics: &mut Diagnostics) -> HashSet<UniqueId> {
    let mut removals = HashSet::new();

    for object in manifest.iter_objects() {
        if let Object::FieldSet(field_set) = object {
            for field in &field_set.fields {
                if let Some(conversion) = field.field_conversion.as_ref() {
                    let target_object = search_object(manifest, &conversion.type_name);

                    match target_object {
                        Some(Object::Enum(target_enum)) => {
                            if field.base_type != target_enum.base_type {
                                diagnostics.add_miette(DifferentBaseTypes {
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
                                match target_enum
                                    .generation_style
                                    .as_ref()
                                    .expect("Generation style has been set here in an earlier pass")
                                {
                                    EnumGenerationStyle::Fallible => {
                                        diagnostics.add_miette(InvalidInfallibleConversion {
                                            conversion: conversion.type_name.span,
                                            context: vec![LabeledSpan::new_with_span(
                                                Some(
                                                    "Target only supports fallible conversion"
                                                        .into(),
                                                ),
                                                target_enum.name.span,
                                            )],
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
                                            diagnostics.add_miette(InvalidInfallibleConversion {
                                                conversion: conversion.type_name.span,
                                                context: vec![
                                                        LabeledSpan::new_with_span(
                                                            Some(format!(
                                                                "The field has a size of {field_bits} bits"
                                                            )),
                                                            field.field_address.span,
                                                        ),
                                                        LabeledSpan::new_with_span(
                                                            Some(format!(
                                                                "Target enum only has a size of {enum_bits} bits. This means not all possible field values can be converted to an enum"
                                                            )),
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
                            if field.base_type != target_extern.base_type {
                                diagnostics.add_miette(DifferentBaseTypes {
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
                                diagnostics.add_miette(InvalidInfallibleConversion {
                                    conversion: conversion.type_name.span,
                                    context: vec![LabeledSpan::new_with_span(
                                        Some("Target only supports fallible conversion".into()),
                                        target_extern.name.span,
                                    )],
                                    existing_type_specifier_content: field
                                        .get_type_specifier_string(),
                                });
                                removals.insert(field.id_with(field_set.id()));
                                continue;
                            }
                        }
                        Some(invalid_object) => {
                            diagnostics.add_miette(ReferencedObjectInvalid {
                                object_reference: conversion.type_name.span,
                                referenced_object: invalid_object.name_span(),
                                help: format!("The referenced object is of type `{}`. But conversions can only reference `enum` and `external` objects.", invalid_object.object_type_name())
                            });
                            removals.insert(field.id_with(field_set.id()));
                            continue;
                        }
                        None => {
                            diagnostics.add_miette(ReferencedObjectDoesNotExist {
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

    removals
}
