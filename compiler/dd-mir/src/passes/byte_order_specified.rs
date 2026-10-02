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
        let mut iter = manifest.iter_objects_with_config_mut();
        while let Some((object, config)) = iter.next() {
            if let Some(fs) = object.as_field_set_mut() {
                if fs.byte_order.is_none() {
                    fs.byte_order = config.byte_order;
                }

                if fs.size_bytes > 1 && fs.byte_order.is_none() {
                    diagnostics.add(UnspecifiedByteOrder {
                        fieldset_name: fs.name.span,
                        properties_span: fs.properties_span,
                    });
                }

                // Even if not required, fill in a byte order so we can always unwrap it later
                fs.byte_order.get_or_insert(ByteOrder::LE);
            }
        }

        Ok(Default::default())
    }
}
