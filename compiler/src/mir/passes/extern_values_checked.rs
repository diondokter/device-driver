use miette::ensure;

use crate::mir::{Manifest, Object};

/// Checks if externs are fully specified
pub fn run_pass(manifest: &mut Manifest) -> miette::Result<()> {
    for object in manifest.iter_objects() {
        let object_name = object.name();

        if let Object::Extern(extern_value) = object {
            ensure!(
                extern_value.base_type.is_fixed_size(),
                "Extern `{object_name}` uses a {} for its base type. Only fixed size integer types are supported for extern objects",
                extern_value.base_type
            );
        }
    }

    Ok(())
}
