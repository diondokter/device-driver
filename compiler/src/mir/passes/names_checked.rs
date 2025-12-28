use crate::mir::{LendingIterator, Manifest, Object};

/// Applies the boundaries to all identifiers and checks the validity
pub fn run_pass(manifest: &mut Manifest) {
    let mut iter = manifest.iter_objects_with_config_mut();
    while let Some((object, config)) = iter.next() {
        if let Object::Device(_) = object {
            // The name rules for devices are slightly different and are done in a different pass
            continue;
        }

        let boundaries = config
            .name_word_boundaries
            .as_deref()
            .unwrap_or(&const { convert_case::Boundary::defaults() });

        if let Err(e) = object
            .name_mut()
            .apply_boundaries(boundaries)
            .check_validity()
        {
            todo!("Emit diagnostic for {e:?}");
        }

        if let Object::FieldSet(field_set) = object {
            for field in &mut field_set.fields {
                if let Err(e) = field.name.apply_boundaries(boundaries).check_validity() {
                    todo!("Emit diagnostic for {e:?}");
                }

                if let Some(conversion) = field.field_conversion.as_mut() {
                    if let Err(e) = conversion
                        .type_name
                        .apply_boundaries(boundaries)
                        .check_validity()
                    {
                        todo!("Emit diagnostic for {e:?}");
                    }
                }
            }
        }

        if let Object::Enum(enum_value) = object {
            for variant in &mut enum_value.variants {
                if let Err(e) = variant.name.apply_boundaries(boundaries).check_validity() {
                    todo!("Emit diagnostic for {e:?}");
                }
            }
        }
    }
}
