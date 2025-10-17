use crate::{
    mir::{BaseType, Integer, LendingIterator, Manifest},
    reporting::{Diagnostics, errors::FieldSizeTooBig},
};

/// Turn all unspecified base types into either bools or uints based on the size of the field
pub fn run_pass(manifest: &mut Manifest, diagnostics: &mut Diagnostics) {
    let mut iter = manifest.iter_objects_with_config_mut();
    while let Some((object, _)) = iter.next() {
        if let Some(field_set) = object.as_field_set_mut() {
            let mut error_fields = Vec::new();

            for (index, field) in field_set.fields.iter_mut().enumerate() {
                let size_bits = field.field_address.len() as u32;
                loop {
                    field.base_type = match field.base_type {
                        BaseType::Unspecified => match size_bits {
                            0 => unreachable!(),
                            1 => BaseType::Bool,
                            _ => BaseType::Uint,
                        },
                        BaseType::Bool => break,
                        BaseType::Uint => match Integer::find_smallest(0, 0, size_bits) {
                            Some(integer) => BaseType::FixedSize(integer),
                            None => {
                                diagnostics.add(FieldSizeTooBig {
                                    field_address: field.field_address.span,
                                    size_bits,
                                });
                                error_fields.push(index);
                                break;
                            }
                        },
                        BaseType::Int => match Integer::find_smallest(-1, 0, size_bits) {
                            Some(integer) => BaseType::FixedSize(integer),
                            None => {
                                diagnostics.add(FieldSizeTooBig {
                                    field_address: field.field_address.span,
                                    size_bits,
                                });
                                error_fields.push(index);
                                break;
                            }
                        },
                        BaseType::FixedSize(_) => break,
                    }
                }
            }

            // Remove the fields with errors
            for error_field in error_fields {
                field_set.fields.remove(error_field);
            }
        }
    }
}
