use miette::{bail, ensure};

use crate::mir::{Enum, Manifest, Object, Repeat, RepeatSource, passes::search_object};

/// Checks if the enums referenced by repeats actually exist
pub fn run_pass(manifest: &mut Manifest) -> miette::Result<()> {
    for object in manifest.iter_objects() {
        let object_name = object.name();
        let object_type = object.type_name();

        if let Some(Repeat {
            source: RepeatSource::Enum(enum_name),
            ..
        }) = object.repeat().as_ref()
        {
            match search_object(manifest, enum_name) {
                Some(Object::Enum(enum_value)) => {
                    ensure!(
                        !enum_has_catch_all(enum_value),
                        "The repeat specified in {object_type} `{object_name}` uses enum `{enum_name}` that has a catch-all. This is not allowed."
                    );
                }
                _ => {
                    bail!(
                        "Cannot find the enum called `{enum_name}` that's used in the repeat specified in {object_type} `{object_name}`"
                    )
                }
            }
        }

        if let Object::FieldSet(fs) = object {
            for field in fs.fields.iter() {
                if let Some(Repeat {
                    source: RepeatSource::Enum(enum_name),
                    ..
                }) = field.repeat.as_ref()
                {
                    match search_object(manifest, enum_name) {
                        Some(Object::Enum(enum_value)) => {
                            ensure!(
                                !enum_has_catch_all(enum_value),
                                "The repeat specified in field `{}` in fieldset `{object_name}` uses enum `{enum_name}` that has a catch-all. This is not allowed.",
                                field.name
                            );
                        }
                        _ => {
                            bail!(
                                "Cannot find the enum called `{enum_name}` that's used in the repeat specified in field `{}` in fieldset `{object_name}`",
                                field.name
                            )
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn enum_has_catch_all(enum_value: &Enum) -> bool {
    enum_value.variants.iter().any(|v| v.value.is_catch_all())
}
