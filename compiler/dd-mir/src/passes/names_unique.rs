use crate::model::{LendingIterator, Manifest, Object, Unique};
use device_driver_diagnostics::{Diagnostics, errors::DuplicateName};

/// Checks if all names are unique to prevent later name collisions.
/// If there is a collision an error is returned.
pub fn run_pass(manifest: &mut Manifest, diagnostics: &mut Diagnostics) {
    // NOT A HASHSET!
    // The hash only looks at the original value of the identifier.
    // We need to use Eq to check the uniqueness of both the original and the split words.
    let mut seen_ids = EqSet::new();
    let mut duplicate_id = 0u32;
    let mut get_duplicate_id = || {
        duplicate_id = duplicate_id.wrapping_add(1);
        duplicate_id
    };

    let mut iter = manifest.iter_objects_with_config_mut();
    while let Some((object, _)) = iter.next() {
        if !seen_ids.insert(object.id()) {
            let original = seen_ids.get(&object.id()).unwrap();
            diagnostics.add_miette(DuplicateName {
                original: original.span(),
                original_value: original.identifier().clone(),
                duplicate: object.id().span(),
                duplicate_value: object.id().identifier().clone(),
            });

            // Duplicate name found. Let's add to the name to make it unique again so it can contribute to later passes
            object.name_mut().set_duplicate_id(get_duplicate_id());
        }

        if let Object::FieldSet(field_set) = object {
            let fs_id = field_set.id();
            for field in field_set.fields.iter_mut() {
                let field_id = field.id_with(fs_id.clone());
                if !seen_ids.insert(field_id.clone()) {
                    let original = seen_ids.get(&field_id).unwrap();
                    diagnostics.add_miette(DuplicateName {
                        original: original.span(),
                        original_value: original.identifier().clone(),
                        duplicate: field_id.span(),
                        duplicate_value: field_id.identifier().clone(),
                    });

                    // Duplicate name found. Let's add to the name to make it unique again so it can contribute to later passes
                    field.name.set_duplicate_id(get_duplicate_id());
                }
            }
        }

        if let Object::Enum(enum_value) = object {
            let e_id = enum_value.id();
            for variant in enum_value.variants.iter_mut() {
                let variant_id = variant.id_with(e_id.clone());
                if !seen_ids.insert(variant_id.clone()) {
                    let original = seen_ids.get(&e_id).unwrap();
                    diagnostics.add_miette(DuplicateName {
                        original: original.span(),
                        original_value: original.identifier().clone(),
                        duplicate: variant_id.span(),
                        duplicate_value: variant_id.identifier().clone(),
                    });

                    // Duplicate name found. Let's add to the name to make it unique again so it can contribute to later passes
                    variant.name.set_duplicate_id(get_duplicate_id());
                }
            }
        }
    }
}

/// Similar to a hashset in API, but uses the [Eq] trait (and linear scan) instead of [Hash]
#[derive(Debug)]
struct EqSet<T: Eq> {
    elements: Vec<T>,
}

impl<T: Eq> EqSet<T> {
    pub const fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    /// Adds a value to the set.
    ///
    /// Returns whether the value was newly inserted. That is:
    ///
    /// - If the set did not previously contain this value, true is returned.
    /// - If the set already contained this value, false is returned, and the set is not modified: original value is not replaced, and the value passed as argument is dropped.
    pub fn insert(&mut self, value: T) -> bool {
        if self.elements.iter().any(|e| e == &value) {
            false
        } else {
            self.elements.push(value);
            true
        }
    }

    /// Returns a reference to the value in the set, if any, that is equal to the given value.
    pub fn get(&self, value: &T) -> Option<&T> {
        self.elements.iter().find(|e| *e == value)
    }
}

#[cfg(test)]
mod tests {
    use convert_case::Boundary;
    use device_driver_common::span::SpanExt;

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
            name: "Device".into_with_dummy_span(),
            device_config: global_config,
            objects: vec![
                Object::Buffer(Buffer {
                    name: "MyBuffer".into_with_dummy_span(),
                    ..Default::default()
                }),
                Object::Buffer(Buffer {
                    name: "MyBuffer".into_with_dummy_span(),
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
            name: "Device".into_with_dummy_span(),
            device_config: global_config,
            objects: vec![Object::FieldSet(FieldSet {
                name: "Reg".into_with_dummy_span(),
                fields: vec![
                    Field {
                        name: "field".into_with_dummy_span(),
                        ..Default::default()
                    },
                    Field {
                        name: "field".into_with_dummy_span(),
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
            name: "Device".into_with_dummy_span(),
            device_config: global_config,
            objects: vec![Object::Enum(Enum {
                name: "Enum".into_with_dummy_span(),
                variants: vec![
                    EnumVariant {
                        name: "Variant".into_with_dummy_span(),
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "Variant".into_with_dummy_span(),
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
