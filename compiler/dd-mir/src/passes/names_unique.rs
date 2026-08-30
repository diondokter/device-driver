use std::{
    collections::{HashMap, HashSet},
    num::NonZeroU32,
};

use crate::{
    model::{Id, LendingIterator, Manifest, Object, ObjectId, ObjectWords},
    passes::{Assumption, Pass},
};
use device_driver_common::identifier::RuntimeNamespace;
use device_driver_diagnostics::{Diagnostics, DynError, errors::DuplicateName};

/// Checks if all names are unique to prevent later name collisions.
/// If there is a collision an error is returned.
pub struct NamesUnique;

impl Pass for NamesUnique {
    const ASSUMPTIONS_MADE: &[Assumption] = &[Assumption::NamesValid];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[Assumption::NamesUnique];

    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<ObjectId>, DynError> {
        let mut namespace_seen_ids =
            HashMap::<RuntimeNamespace, (HashSet<ObjectId>, HashSet<ObjectWords>)>::new();

        let mut duplicate_id = 0u32;
        let mut get_duplicate_id = || {
            duplicate_id = duplicate_id.wrapping_add(1);
            NonZeroU32::new(duplicate_id).ok_or_else(|| DynError::new("got too many duplicates"))
        };

        let mut iter = manifest.iter_objects_with_config_mut();
        while let Some((object, _)) = iter.next() {
            let object_id = object.id();

            for object_id in object_id.concrete_namespace_ids()? {
                let object_id = object_id?;
                let namespace = *object_id.identifier().namespace();

                let (seen_ids, seen_words) = namespace_seen_ids.entry(namespace).or_default();
                if !seen_ids.insert(object_id.clone()) | !seen_words.insert(object_id.words()) {
                    let original = seen_ids.get(&object_id).unwrap();
                    diagnostics.add(DuplicateName {
                        original: original.span(),
                        original_value: original.identifier().clone(),
                        duplicate: object_id.span(),
                        duplicate_value: object_id.identifier().clone(),
                    });

                    // Duplicate name found. Let's add to the name to make it unique again so it can contribute to later passes
                    object.name_mut().set_duplicate_id(get_duplicate_id()?);
                }
            }

            if let Object::FieldSet(field_set) = object {
                let mut seen_ids = HashSet::new();
                let mut seen_words = HashSet::new();
                for field in field_set.fields.iter_mut() {
                    let field_id = field.id();
                    if !seen_ids.insert(field_id.clone()) | !seen_words.insert(field_id.words()) {
                        let original = seen_ids.get(&field_id).unwrap();
                        diagnostics.add(DuplicateName {
                            original: original.span(),
                            original_value: original.identifier().clone(),
                            duplicate: field_id.span(),
                            duplicate_value: field_id.identifier().clone(),
                        });

                        // Duplicate name found. Let's add to the name to make it unique again so it can contribute to later passes
                        field.name.set_duplicate_id(get_duplicate_id()?);
                    }
                }
            }

            if let Object::Enum(enum_value) = object {
                let mut seen_ids = HashSet::new();
                let mut seen_words = HashSet::new();
                for variant in enum_value.variants.iter_mut() {
                    let variant_id = variant.id();
                    if !seen_ids.insert(variant_id.clone()) | !seen_words.insert(variant_id.words())
                    {
                        let original = seen_ids.get(&variant_id).unwrap();
                        diagnostics.add(DuplicateName {
                            original: original.span(),
                            original_value: original.identifier().clone(),
                            duplicate: variant_id.span(),
                            duplicate_value: variant_id.identifier().clone(),
                        });

                        // Duplicate name found. Let's add to the name to make it unique again so it can contribute to later passes
                        variant.name.set_duplicate_id(get_duplicate_id()?);
                    }
                }
            }
        }

        Ok(Default::default())
    }
}

#[cfg(test)]
mod tests {
    use convert_case::Boundary;
    use device_driver_common::{identifier::Identifier, span::SpanExt};

    use crate::model::{Buffer, Device, DeviceConfig, Enum, EnumVariant, Field, FieldSet, Object};

    use super::*;

    #[test]
    fn object_names_not_unique() {
        let global_config = DeviceConfig {
            name_word_boundaries: Some(Boundary::defaults_from("-")),
            ..Default::default()
        };

        let mut start_mir = Device {
            description: String::new(),
            name: Identifier::try_parse("Device").unwrap().with_dummy_span(),
            device_config: global_config,
            objects: vec![
                Object::Buffer(Buffer {
                    name: Identifier::try_parse("MyBuffer").unwrap().with_dummy_span(),
                    ..Default::default()
                }),
                Object::Buffer(Buffer {
                    name: Identifier::try_parse("MyBuffer").unwrap().with_dummy_span(),
                    ..Default::default()
                }),
            ],
            ..Default::default()
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        NamesUnique::run_pass(&mut start_mir, &mut diagnostics).unwrap();
        assert!(diagnostics.has_error())
    }

    #[test]
    fn field_names_not_unique() {
        let global_config = DeviceConfig {
            name_word_boundaries: Some(Boundary::defaults_from("-")),
            ..Default::default()
        };

        let mut start_mir = Device {
            description: String::new(),
            name: Identifier::try_parse("Device").unwrap().with_dummy_span(),
            device_config: global_config,
            objects: vec![Object::FieldSet(FieldSet {
                name: Identifier::try_parse("Reg").unwrap().with_dummy_span(),
                fields: vec![
                    Field {
                        name: Identifier::try_parse("field").unwrap().with_dummy_span(),
                        ..Default::default()
                    },
                    Field {
                        name: Identifier::try_parse("field").unwrap().with_dummy_span(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            })],
            ..Default::default()
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        NamesUnique::run_pass(&mut start_mir, &mut diagnostics).unwrap();
        assert!(diagnostics.has_error())
    }

    #[test]
    fn duplicate_generated_enum_variants() {
        let global_config = DeviceConfig {
            name_word_boundaries: Some(Boundary::defaults_from("-")),
            ..Default::default()
        };

        let mut start_mir = Device {
            description: String::new(),
            name: Identifier::try_parse("Device").unwrap().with_dummy_span(),
            device_config: global_config,
            objects: vec![Object::Enum(Enum {
                name: Identifier::try_parse("Enum").unwrap().with_dummy_span(),
                variants: vec![
                    EnumVariant {
                        name: Identifier::try_parse("Variant").unwrap().with_dummy_span(),
                        ..Default::default()
                    },
                    EnumVariant {
                        name: Identifier::try_parse("Variant").unwrap().with_dummy_span(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            })],
            ..Default::default()
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        NamesUnique::run_pass(&mut start_mir, &mut diagnostics).unwrap();
        assert!(diagnostics.has_error())
    }
}
