use std::{collections::HashSet, num::NonZero};

use crate::{
    model::{LendingIterator, Manifest, Object, ObjectId},
    passes::{Assumption, Pass},
};
use device_driver_diagnostics::{Diagnostics, DynError};

/// Goes through all identifiers with a local namespace and sets its definition site
pub struct LocalNamespacesAssigned;

impl Pass for LocalNamespacesAssigned {
    const ASSUMPTIONS_MADE: &[Assumption] = &[];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[Assumption::LocalNamespacesAssigned];

    fn run_pass(
        manifest: &mut Manifest,
        _diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<ObjectId>, DynError> {
        let mut next_size_id = NonZero::new(1).unwrap();

        let mut iter = manifest.iter_objects_with_config_mut();
        while let Some((object, _)) = iter.next() {
            if let Object::FieldSet(fs) = object {
                for field in fs.fields.iter_mut() {
                    field.name.set_local_site(next_size_id);
                    next_size_id = next_size_id
                        .checked_add(1)
                        .ok_or_else(|| DynError::new("too many local sites"))?;
                }
            }
            if let Object::Enum(e) = object {
                for variant in e.variants.iter_mut() {
                    variant.name.set_local_site(next_size_id);
                    next_size_id = next_size_id
                        .checked_add(1)
                        .ok_or_else(|| DynError::new("too many local sites"))?;
                }
            }
        }

        Ok(Default::default())
    }
}
