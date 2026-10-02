use std::collections::HashSet;

use crate::{
    model::{FieldsetRef, Manifest, ObjectId},
    passes::{Assumption, Pass},
};
use device_driver_diagnostics::{Diagnostics, DynError, errors::InvalidFieldsetRef};

/// Checks whether all registers and commands point to existing fieldsets
pub struct FieldsetRefsValid;

impl Pass for FieldsetRefsValid {
    const ASSUMPTIONS_MADE: &[Assumption] = &[Assumption::NamesUnique];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[Assumption::FieldsetRefsValid];

    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<ObjectId>, DynError> {
        let mut removals = HashSet::new();

        for (object_id, object) in manifest.objects_enumerated() {
            let fieldset_refs = object.fieldset_refs();

            for fieldset_ref in fieldset_refs {
                if manifest.search_fieldset(&fieldset_ref).is_some() {
                    continue;
                }

                // We could not find the fieldset.
                // If the ref was an id, it was simply removed by another pass alread.
                // But if it's an identifier ref, then maybe there's a typo or it points to an object of the wrong type.
                // In that case we should create a diagnostic

                let id_ref = match fieldset_ref.value {
                    FieldsetRef::Identifier(identifier_ref) => identifier_ref,
                    FieldsetRef::Id(_) => {
                        continue;
                    }
                };

                diagnostics.add(InvalidFieldsetRef {
                    reference: fieldset_ref.span,
                    pointee: manifest
                        .search_object(&id_ref)
                        .map(|found_object| found_object.name_span()),
                });

                removals.insert(object_id);
                break;
            }
        }

        Ok(removals)
    }
}
