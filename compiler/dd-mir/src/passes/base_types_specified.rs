use crate::model::{LendingIterator, Manifest};
use device_driver_common::specifiers::{BaseType, Integer};
use device_driver_diagnostics::{Diagnostics, errors::IntegerFieldSizeTooBig};

/// Turn all unspecified base types into either bools or uints based on the size of the field
pub fn run_pass(manifest: &mut Manifest, diagnostics: &mut Diagnostics) {
    let mut iter = manifest.iter_objects_with_config_mut();
    while let Some((object, _)) = iter.next() {
        if let Some(field_set) = object.as_field_set_mut() {
            for field in &mut field_set.fields {
                loop {
                    let size_bits = field.field_address.len() as u32;
                    field.base_type.value = match field.base_type.value {
                        BaseType::Unspecified => match size_bits {
                            0 => unreachable!(),
                            1 => BaseType::Bool,
                            _ => BaseType::Uint,
                        },
                        BaseType::Bool => break,
                        BaseType::Uint => {
                            if let Some(integer) = Integer::find_smallest(0, 0, size_bits) {
                                BaseType::FixedSize(integer)
                            } else {
                                diagnostics.add(IntegerFieldSizeTooBig {
                                    field_address: field.field_address.span,
                                    size_bits,
                                    base_type: field.base_type.span.or(field.name.span),
                                    field_set: field_set.name.span,
                                });
                                // Fix the size for now so we can continue using this field later
                                field.field_address.end = field.field_address.start + 64;
                                continue;
                            }
                        }
                        BaseType::Int => {
                            if let Some(integer) = Integer::find_smallest(-1, 0, size_bits) {
                                BaseType::FixedSize(integer)
                            } else {
                                diagnostics.add(IntegerFieldSizeTooBig {
                                    field_address: field.field_address.span,
                                    size_bits,
                                    base_type: field.base_type.span.or(field.name.span),
                                    field_set: field_set.name.span,
                                });
                                // Fix the size for now so we can continue using this field later
                                field.field_address.end = field.field_address.start + 64;
                                continue;
                            }
                        }
                        BaseType::FixedSize(_) => break,
                    }
                }
            }
        }
    }
}
