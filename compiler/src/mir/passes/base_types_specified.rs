use crate::{
    mir::{BaseType, Integer, LendingIterator, Manifest},
    reporting::{Diagnostics, errors::FieldSizeTooBig},
};

/// Turn all unspecified base types into either bools or uints based on the size of the field
pub fn run_pass(manifest: &mut Manifest, diagnostics: &mut Diagnostics) {
    let mut iter = manifest.iter_objects_with_config_mut();
    while let Some((object, _)) = iter.next() {
        if let Some(field_set) = object.as_field_set_mut() {
            for field in field_set.fields.iter_mut() {
                loop {
                    let size_bits = field.field_address.len() as u32;
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
                                // Fix the size for now so we can continue using this field later
                                field.field_address.end = field.field_address.start + 64;
                                continue;
                            }
                        },
                        BaseType::Int => match Integer::find_smallest(-1, 0, size_bits) {
                            Some(integer) => BaseType::FixedSize(integer),
                            None => {
                                diagnostics.add(FieldSizeTooBig {
                                    field_address: field.field_address.span,
                                    size_bits,
                                });
                                // Fix the size for now so we can continue using this field later
                                field.field_address.end = field.field_address.start + 64;
                                continue;
                            }
                        },
                        BaseType::FixedSize(_) => break,
                    }
                }
            }
        }
    }
}
