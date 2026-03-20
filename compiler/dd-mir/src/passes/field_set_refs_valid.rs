use std::collections::HashSet;

use crate::{
    model::{Manifest, Object, Unique, UniqueId},
    search_object,
};
use device_driver_diagnostics::{Diagnostics, errors::InvalidFieldsetRef};

/// Checks whether all registers and commands point to existing fieldsets
pub fn run_pass(manifest: &mut Manifest, diagnostics: &mut Diagnostics) -> HashSet<UniqueId> {
    let mut removals = HashSet::new();

    for object in manifest.iter_objects() {
        let fieldset_refs = object.fieldset_refs();

        for fieldset_ref in fieldset_refs {
            let pointee = match search_object(manifest, &fieldset_ref) {
                Some(Object::FieldSet(_)) => continue,
                Some(found_object) => Some(found_object.name_span()),
                None => None,
            };

            diagnostics.add(InvalidFieldsetRef {
                reference: fieldset_ref.span,
                pointee,
            });

            removals.insert(object.id());
            break;
        }
    }

    removals
}
