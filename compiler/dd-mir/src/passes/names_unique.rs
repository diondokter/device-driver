use std::{
    collections::{HashMap, HashSet},
    num::NonZeroU32,
    sync::Arc,
};

use crate::{
    model::{Manifest, Object, ObjectId},
    passes::{Assumption, Pass},
};
use device_driver_common::{
    identifier::{Identifier, RuntimeNamespace},
    interner::Istr,
};
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
        let mut namespace_seen_names =
            HashMap::<RuntimeNamespace, (HashMap<Istr, ObjectId>, HashSet<Arc<[String]>>)>::new();

        let mut duplicates = Vec::new();

        for (object_id, object) in manifest.objects_enumerated() {
            for object_namespace in object.name().namespace().concrete_namespaces() {
                let (seen_originals, seen_words) =
                    namespace_seen_names.entry(object_namespace).or_default();

                if seen_originals
                    .insert(object.name().original(), object_id)
                    .is_some()
                    || !seen_words.insert(object.name().words().clone())
                {
                    let original_id = seen_originals.get(&object.name().original()).unwrap();
                    let original = manifest.object(*original_id).unwrap();
                    diagnostics.add(DuplicateName {
                        original: original.span(),
                        original_value: original.name().clone(),
                        duplicate: object.span(),
                        duplicate_value: object.name().clone(),
                    });

                    // Duplicate name found. Let's add to the name to make it unique again so it can contribute to later passes
                    duplicates.push(object_id);
                }
            }
        }

        for (i, duplicate) in duplicates.into_iter().enumerate() {
            manifest
                .object_mut(duplicate)
                .unwrap()
                .name_mut()
                .set_duplicate_id(NonZeroU32::new((i + 1) as u32).unwrap());
        }

        Ok(Default::default())
    }
}
