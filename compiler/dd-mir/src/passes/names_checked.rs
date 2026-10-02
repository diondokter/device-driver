use std::collections::HashSet;

use crate::{
    model::{Manifest, ObjectId, ObjectType},
    passes::{Assumption, Pass},
};
use device_driver_diagnostics::{Diagnostics, DynError, errors::InvalidIdentifier};

/// Applies the boundaries to all identifiers and checks the validity
pub struct NamesChecked;

impl Pass for NamesChecked {
    const ASSUMPTIONS_MADE: &[Assumption] = &[Assumption::LocalNamespacesAssigned];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[Assumption::NamesValid];

    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<ObjectId>, DynError> {
        let mut removals = HashSet::new();

        let all_objects = manifest.object_ids().collect::<Vec<_>>();

        for object_id in all_objects {
            if object_id.object_type() == ObjectType::Device {
                // The name rules for devices are slightly different and are done in a different pass
                continue;
            }

            let config = manifest.object_config(object_id);

            let boundaries = config
                .name_word_boundaries
                .as_deref()
                .unwrap_or(&const { convert_case::Boundary::defaults() })
                .iter()
                .copied()
                .collect::<Vec<_>>();

            let mut object = manifest.object_mut(object_id).unwrap();

            if let Err(e) = object
                .name_mut()
                .apply_boundaries(&boundaries)
                .check_validity()
            {
                diagnostics.add(InvalidIdentifier::new(e, object.as_ref().name_span()));
                removals.insert(object_id);
                continue;
            }
        }

        Ok(removals)
    }
}
