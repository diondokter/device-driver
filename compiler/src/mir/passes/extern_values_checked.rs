use std::collections::HashSet;

use crate::{
    mir::{Manifest, Object, Unique, UniqueId},
    reporting::{Diagnostics, errors::ExternInvalidBaseType},
};

/// Checks if externs are fully specified
pub fn run_pass(manifest: &mut Manifest, diagnostics: &mut Diagnostics) -> HashSet<UniqueId> {
    let mut removals = HashSet::new();

    for object in manifest.iter_objects() {
        if let Object::Extern(extern_value) = object
            && !extern_value.base_type.is_fixed_size()
        {
            diagnostics.add(ExternInvalidBaseType {
                extern_name: extern_value.name.span,
                base_type: (!extern_value.base_type.is_unspecified())
                    .then_some(extern_value.base_type.span),
            });
            removals.insert(extern_value.id());
            continue;
        }
    }

    removals
}
