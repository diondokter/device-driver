use std::collections::HashSet;

use crate::{
    model::{Manifest, ObjectId},
    passes::{Assumption, Pass},
};
use device_driver_common::specifiers::ByteOrder;
use device_driver_diagnostics::{Diagnostics, DynError, errors::UnspecifiedByteOrder};

/// Checks if the byte order is set for all registers and commands that need it and fills it out for the ones that aren't specified
pub struct ByteOrderSpecified;

impl Pass for ByteOrderSpecified {
    const ASSUMPTIONS_MADE: &[Assumption] = &[];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[Assumption::ByteOrderSpecified];

    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<ObjectId>, DynError> {
        let mut fieldset_changes = Vec::new();

        for (fs_id, fs) in manifest.fieldsets.iter_enumerated() {
            let config = manifest.object_config(fs_id);

            if fs.size_bytes > 1 && fs.byte_order.is_none() && config.byte_order.is_none() {
                diagnostics.add(UnspecifiedByteOrder {
                    fieldset_name: fs.name.span,
                    properties_span: fs.properties_span,
                });
            }

            if fs.byte_order.is_none() {
                // Even if not required, fill in a byte order so we can always unwrap it later
                fieldset_changes.push((fs_id, config.byte_order.unwrap_or(ByteOrder::LE)));
            }
        }

        for (fs_id, target_byte_order) in fieldset_changes {
            manifest.fieldsets.get_mut(fs_id).unwrap().byte_order = Some(target_byte_order);
        }

        Ok(Default::default())
    }
}
