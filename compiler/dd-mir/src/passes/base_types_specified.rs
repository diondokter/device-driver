use std::collections::{HashMap, HashSet};

use crate::{
    model::{LendingIterator, Manifest, Object, UniqueId},
    passes::Pass,
};
use device_driver_common::specifiers::{BaseType, Integer};
use device_driver_diagnostics::{Diagnostics, DynError, errors::IntegerFieldSizeTooBig};

/// Turn all unspecified base types into either bools or uints based on the size of the field
pub struct BaseTypesSpecified;

impl Pass for BaseTypesSpecified {
    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<UniqueId>, DynError> {
        // Collect base types of all objects since we can't later in the pass because of the mut borrow of manifest
        let base_types = manifest
            .iter_objects()
            .filter_map(|object| match object {
                Object::Enum(e) => Some((e.name.take_ref(), e.base_type)),
                Object::Extern(e) => Some((e.name.take_ref(), e.base_type)),
                _ => None,
            })
            .collect::<HashMap<_, _>>();

        let mut iter = manifest.iter_objects_with_config_mut();
        while let Some((object, _)) = iter.next() {
            if let Some(field_set) = object.as_field_set_mut() {
                for field in &mut field_set.fields {
                    loop {
                        let size_bits = field.field_address.len();
                        field.base_type.value = match field.base_type.value {
                            BaseType::Unspecified => {
                                match field.field_conversion.as_ref().and_then(|conversion| {
                                    base_types.get(&conversion.type_name.value)
                                }) {
                                    Some(conversion_base_type) => conversion_base_type.value,
                                    None => {
                                        // No conversion type? Then base it off of the size bits
                                        //
                                        // We can also get here if a conversion was specified, but the base type couldn't be determined.
                                        // In that case we don't need to do anything special here because a later pass will turn it into an error.
                                        match size_bits {
                                            0 => unreachable!(),
                                            1 => BaseType::Bool,
                                            _ => BaseType::Uint,
                                        }
                                    }
                                }
                            }
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
                                    field.field_address.end = field.field_address.start + 63;
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

        Ok(Default::default())
    }
}
