use std::collections::HashSet;

use crate::{
    model::{Manifest, Object, ObjectId},
    passes::{Assumption, Pass},
};
use device_driver_common::specifiers::BaseType;
use device_driver_diagnostics::{Diagnostics, DynError, errors::BoolFieldTooLarge};

/// Check all bool fields. They must be exactly zero or one bits
pub struct BoolFieldsChecked;

impl Pass for BoolFieldsChecked {
    const ASSUMPTIONS_MADE: &[Assumption] = &[Assumption::FieldBaseTypesSpecified];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[];

    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<ObjectId>, DynError> {
        let mut fixups = Vec::new();

        for (field_id, field) in manifest.fields.iter_enumerated() {
            if field.base_type == BaseType::Bool && field.field_address.len() != 1 {
                let Some(fieldset) = manifest.object_parents(field_id).last() else {
                    continue;
                };
                let Some(Object::FieldSet(fieldset)) = manifest.object(*fieldset) else {
                    continue;
                };

                diagnostics.add(BoolFieldTooLarge {
                    base_type: if field.base_type.span.is_empty() {
                        None
                    } else {
                        Some(field.base_type.span)
                    },
                    address: field.field_address.span,
                    address_bits: field.field_address.len() as u32,
                    address_start: field.field_address.start,

                    field_set_context: fieldset.name.span,
                });

                fixups.push(field_id);
            }
        }

        for field_id in fixups {
            let field = manifest.fields.get_mut(field_id).unwrap();
            // To fix for further use, set the len to just 1
            field.field_address.end = field.field_address.start;
        }

        Ok(Default::default())
    }
}
