use std::{collections::HashSet, num::NonZero};

use crate::{
    model::{Manifest, ObjectId},
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
        let mut next_site_id = NonZero::new(1).unwrap();

        for fieldset in manifest.fieldsets.iter() {
            for field_id in fieldset.fields.iter() {
                if let Some(field) = manifest.fields.get_mut(*field_id) {
                    field.name.set_local_site(next_site_id);
                }
            }
            next_site_id = next_site_id
                .checked_add(1)
                .ok_or_else(|| DynError::new("too many local sites"))?;
        }

        for enum_value in manifest.enums.iter() {
            for variant_id in enum_value.variants.iter() {
                if let Some(variant) = manifest.enum_variants.get_mut(*variant_id) {
                    variant.name.set_local_site(next_site_id);
                }
            }
            next_site_id = next_site_id
                .checked_add(1)
                .ok_or_else(|| DynError::new("too many local sites"))?;
        }

        Ok(Default::default())
    }
}
