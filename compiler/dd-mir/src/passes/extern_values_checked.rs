use std::collections::HashSet;

use crate::{
    model::{Manifest, Object, Unique, UniqueId},
    passes::Pass,
};
use device_driver_diagnostics::{
    Diagnostics, DynError,
    errors::{ExternInvalidBaseType, ExternInvalidSizeBits},
};

/// Checks if externs are fully specified
pub struct ExternValuesChecked;

impl Pass for ExternValuesChecked {
    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<UniqueId>, DynError> {
        let mut removals = HashSet::new();

        for object in manifest.iter_objects() {
            if let Object::Extern(extern_value) = object {
                match extern_value.base_type.as_fixed_size() {
                    Some(base_type) => {
                        if let Some(size_bits) = extern_value.size_bits
                            && size_bits.value > base_type.size_bits().into()
                        {
                            diagnostics.add(ExternInvalidSizeBits {
                                extern_name: extern_value.name.span,
                                size_bits: size_bits.span,
                                reason: "value is bigger than the base type size".into(),
                            });
                            removals.insert(extern_value.id());
                            continue;
                        }
                    }
                    None => {
                        diagnostics.add(ExternInvalidBaseType {
                            extern_name: extern_value.name.span,
                            base_type: (!extern_value.base_type.span.is_empty())
                                .then_some(extern_value.base_type.span),
                        });
                        removals.insert(extern_value.id());
                        continue;
                    }
                }
            }
        }

        Ok(removals)
    }
}
