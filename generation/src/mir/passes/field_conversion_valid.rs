use anyhow::{bail, ensure};

use crate::mir::{Device, EnumGenerationStyle, Object, passes::recurse_objects};

/// Checks if fields that have conversion and specified no try to be used, are valid in doing so
pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    recurse_objects(&device.objects, &mut |object| {
        if let Object::FieldSet(field_set) = object {
            for field in field_set.fields.iter() {
                if let Some(conversion) = field.field_conversion.as_ref() {
                    if !conversion.use_try {
                        // Check if we know the value we're converting to and if we can support non-try conversion
                        let target_object =
                            super::search_object(&device.objects, &conversion.type_name);

                        match target_object {
                            Some(Object::Enum(target_enum)) => {
                                match target_enum
                                    .generation_style
                                    .as_ref()
                                    .expect("Generation style has been set here in an earlier pass")
                                {
                                    EnumGenerationStyle::Fallible => {
                                        bail!(
                                            "Field `{}` of FieldSet `{}` uses an infallible conversion for an enum that only has fallible conversion. Try adding a '?' to mark the conversion as fallible",
                                            field.name,
                                            field_set.name
                                        );
                                    }
                                    EnumGenerationStyle::InfallibleWithinRange => {
                                        let field_bits = field.field_address.len() as u32;
                                        let enum_bits = target_enum.size_bits.expect(
                                            "Enum size_bits is already set in a previous pass",
                                        );

                                        ensure!(
                                            field_bits == enum_bits,
                                            "Field `{}` of FieldSet `{}` uses an infallible conversion for an enum of {enum_bits} bits. The field is {field_bits} large and thus infallible conversion is not possible",
                                            field.name,
                                            field_set.name
                                        )
                                    }
                                    EnumGenerationStyle::Fallback => {
                                        // This always works
                                    }
                                }
                            }
                            Some(_) => bail!(
                                "Field `{}` of FieldSet `{}` specifies a conversion to type: `{}`. This is not an enum or external type and is thus not allowed",
                                field.name,
                                field_set.name,
                                conversion.type_name
                            ),
                            None => {
                                bail!(
                                    "Field `{}` of FieldSet `{}` specifies a conversion to type: `{}`. This is type is unknown",
                                    field.name,
                                    field_set.name,
                                    conversion.type_name
                                )
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    })
}
