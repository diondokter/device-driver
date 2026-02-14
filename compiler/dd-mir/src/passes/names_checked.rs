use std::collections::HashSet;

use crate::model::{LendingIterator, Manifest, Object, Unique, UniqueId};
use device_driver_diagnostics::{Diagnostics, errors::InvalidIdentifier};

/// Applies the boundaries to all identifiers and checks the validity
pub fn run_pass(manifest: &mut Manifest, diagnostics: &mut Diagnostics) -> HashSet<UniqueId> {
    let mut removals = HashSet::new();

    let mut iter = manifest.iter_objects_with_config_mut();
    while let Some((object, config)) = iter.next() {
        if let Object::Device(_) = object {
            // The name rules for devices are slightly different and are done in a different pass
            continue;
        }

        let boundaries = config
            .name_word_boundaries
            .as_deref()
            .unwrap_or(&const { convert_case::Boundary::defaults() });

        if let Err(e) = object
            .name_mut()
            .apply_boundaries(boundaries)
            .check_validity()
        {
            diagnostics.add(InvalidIdentifier::new(e, object.name_span()));
            removals.insert(object.id());
            continue;
        }

        if let Object::FieldSet(field_set) = object {
            let mut field_removals = HashSet::new();
            let field_set_id = field_set.id();
            for field in &mut field_set.fields {
                if let Err(e) = field.name.apply_boundaries(boundaries).check_validity() {
                    diagnostics.add(InvalidIdentifier::new(e, field.name.span));
                    field_removals.insert(field.id_with(field_set_id.clone()));
                }
            }

            field_set
                .fields
                .retain(|field| !field_removals.contains(&field.id_with(field_set_id.clone())));
        }

        if let Object::Enum(enum_value) = object {
            let mut variant_removals = HashSet::new();
            let enum_id = enum_value.id();
            for variant in &mut enum_value.variants {
                if let Err(e) = variant.name.apply_boundaries(boundaries).check_validity() {
                    diagnostics.add(InvalidIdentifier::new(e, variant.name.span));
                    variant_removals.insert(variant.id_with(enum_id.clone()));
                }
            }

            enum_value
                .variants
                .retain(|variant| !variant_removals.contains(&variant.id_with(enum_id.clone())));
        }
    }

    removals
}
