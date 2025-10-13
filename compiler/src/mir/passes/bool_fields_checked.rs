use crate::mir::{BaseType, LendingIterator, Manifest};
use miette::ensure;

/// Check all bool fields. They must be exactly zero or one bits long and have no conversion
pub fn run_pass(manifest: &mut Manifest) -> miette::Result<()> {
    let mut iter = manifest.iter_objects_with_config_mut();
    while let Some((object, _)) = iter.next() {
        let object_name = object.name().to_string();

        for field in object
            .as_field_set_mut()
            .into_iter()
            .flat_map(|fs| &mut fs.fields)
        {
            if field.base_type == BaseType::Bool {
                // When zero bits long, extend to one bit
                if field.field_address.start == field.field_address.end {
                    field.field_address.end += 1;
                }

                ensure!(
                    field.field_address.clone().count() == 1,
                    "Fieldset `{}` has field `{}` which is of base type `bool` and is larger than 1 bit. A bool can only be zero or one bit.",
                    object_name,
                    field.name
                );

                ensure!(
                    field.field_conversion.is_none(),
                    "Fieldset `{}` has field `{}` which is of base type `bool` and has specified a conversion. This is not supported for bools.",
                    object_name,
                    field.name
                );
            }
        }
    }

    Ok(())
}
