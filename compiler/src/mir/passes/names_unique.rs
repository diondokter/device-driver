use std::{
    collections::HashSet,
    hash::{DefaultHasher, Hash, Hasher},
};

use convert_case::Boundary;

use crate::{
    identifier::Identifier,
    mir::{Enum, LendingIterator, Manifest, Object, Unique},
    reporting::{Diagnostics, errors::DuplicateName},
};

/// Checks if all names are unique to prevent later name collisions.
/// If there is a collision an error is returned.
pub fn run_pass(manifest: &mut Manifest, diagnostics: &mut Diagnostics) {
    let mut seen_ids = HashSet::new();

    let mut object_index = 0;
    let mut iter = manifest.iter_objects_with_config_mut();
    while let Some((object, _)) = iter.next() {
        if !seen_ids.insert(object.id()) {
            diagnostics.add(DuplicateName {
                original: seen_ids.get(&object.id()).unwrap().span(),
                duplicate: object.id().span(),
            });

            // Duplicate name found. Let's add to the name to make it unique again so it can contribute to later passes
            let extension = get_extension(object, object_index);
            *object.name_mut() = object.name_mut().clone().concat(&extension);
        }

        if let Object::FieldSet(field_set) = object {
            let fs_id = field_set.id();
            for (field_index, field) in field_set.fields.iter_mut().enumerate() {
                let field_id = field.id_with(fs_id.clone());
                if !seen_ids.insert(field_id.clone()) {
                    diagnostics.add(DuplicateName {
                        original: seen_ids.get(&field_id).unwrap().span(),
                        duplicate: field_id.span(),
                    });

                    // Duplicate name found. Let's add to the name to make it unique again so it can contribute to later passes
                    let extension = get_extension(field, field_index);
                    field.name.value = field.name.value.clone().concat(&extension);
                }
            }
        }

        if let Object::Enum(Enum { variants, .. }) = object {
            let mut seen_variant_names = HashSet::new();

            for (variant_index, variant) in variants.iter_mut().enumerate() {
                if !seen_variant_names.insert(variant.id()) {
                    diagnostics.add(DuplicateName {
                        original: seen_variant_names.get(&variant.id()).unwrap().span(),
                        duplicate: variant.id().span(),
                    });

                    // Duplicate name found. Let's add to the name to make it unique again so it can contribute to later passes
                    let extension = get_extension(variant, variant_index);
                    variant.name.value = variant.name.value.clone().concat(&extension);
                }
            }
        }

        object_index += 1;
    }
}

fn get_extension(val: &impl Hash, index: usize) -> Identifier {
    let mut hasher = DefaultHasher::new();
    val.hash(&mut hasher);
    index.hash(&mut hasher);
    let mut id = Identifier::try_parse(&format!("dup_{:016x}", hasher.finish())).unwrap();
    id.apply_boundaries(&[Boundary::Underscore]);
    id
}

#[cfg(test)]
mod tests {
    use convert_case::Boundary;

    use crate::mir::{Buffer, Device, DeviceConfig, EnumVariant, Field, FieldSet, Object};

    use super::*;

    #[test]
    fn object_names_not_unique() {
        let global_config = DeviceConfig {
            name_word_boundaries: Some(Boundary::defaults_from("-")),
            ..Default::default()
        };

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into(),
            device_config: global_config,
            objects: vec![
                Object::Buffer(Buffer {
                    name: "MyBuffer".into(),
                    ..Default::default()
                }),
                Object::Buffer(Buffer {
                    name: "MyBuffer".into(),
                    ..Default::default()
                }),
            ],
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
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
            name: "Device".into(),
            device_config: global_config,
            objects: vec![Object::FieldSet(FieldSet {
                name: "Reg".into(),
                fields: vec![
                    Field {
                        name: "field".into(),
                        ..Default::default()
                    },
                    Field {
                        name: "field".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            })],
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
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
            name: "Device".into(),
            device_config: global_config,
            objects: vec![Object::Enum(Enum {
                name: "Enum".into(),
                variants: vec![
                    EnumVariant {
                        name: "Variant".into(),
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "Variant".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            })],
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
        assert!(diagnostics.has_error())
    }
}
