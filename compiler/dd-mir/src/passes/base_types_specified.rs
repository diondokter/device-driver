use std::collections::{HashMap, HashSet};

use crate::{
    model::{Manifest, Object, ObjectId, TypeRef},
    passes::{Assumption, Pass},
};
use device_driver_common::specifiers::{BaseType, Integer};
use device_driver_diagnostics::{Diagnostics, DynError, errors::IntegerFieldSizeTooBig};

/// Turn all unspecified base types into either bools or uints based on the size of the field
pub struct BaseTypesSpecified;

impl Pass for BaseTypesSpecified {
    const ASSUMPTIONS_MADE: &[Assumption] = &[
        Assumption::ExternBaseTypesSpecified,
        Assumption::EnumBaseTypesSpecified,
    ];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[Assumption::FieldBaseTypesSpecified];

    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<ObjectId>, DynError> {
        // Collect base types of all objects since we can't later in the pass because of the mut borrow of manifest
        let base_types = manifest
            .objects()
            .filter_map(|object| match object {
                Object::Enum(e) => Some((e.name.take_ref(), e.base_type)),
                Object::Extern(e) => Some((e.name.take_ref(), e.base_type)),
                _ => None,
            })
            .collect::<HashMap<_, _>>();

        for field_id in manifest.fields.ids().collect::<Vec<_>>() {
            loop {
                let field = manifest.fields.get(field_id).unwrap();

                let size_bits = field.field_address.len();
                let new_base_type = match field.base_type.value {
                    BaseType::Unspecified => {
                        match field.field_conversion.as_ref().and_then(|conversion| {
                            base_types.get(&match &conversion.type_ref.value {
                                TypeRef::Identifier(identifier_ref) => identifier_ref.clone(),
                                TypeRef::Id(object_id) => object_id
                                    .get(manifest)
                                    .unwrap()
                                    .name()
                                    .clone()
                                    .cast_assert()
                                    .take_ref(),
                            })
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
                            let Some(parent_fs_id) = manifest.object_parents(field_id).last()
                            else {
                                continue;
                            };
                            let Some(Object::FieldSet(parent_fs)) = manifest.object(*parent_fs_id)
                            else {
                                continue;
                            };
                            diagnostics.add(IntegerFieldSizeTooBig {
                                field_address: field.field_address.span,
                                size_bits,
                                base_type: field.base_type.span.or(field.name.span),
                                field_set: parent_fs.name.span,
                            });
                            // Fix the size for now so we can continue using this field later
                            manifest.fields.get_mut(field_id).unwrap().field_address.end =
                                field.field_address.start + 63;
                            continue;
                        }
                    }
                    BaseType::Int => {
                        if let Some(integer) = Integer::find_smallest(-1, 0, size_bits) {
                            BaseType::FixedSize(integer)
                        } else {
                            let Some(parent_fs_id) = manifest.object_parents(field_id).last()
                            else {
                                continue;
                            };
                            let Some(Object::FieldSet(parent_fs)) = manifest.object(*parent_fs_id)
                            else {
                                continue;
                            };
                            diagnostics.add(IntegerFieldSizeTooBig {
                                field_address: field.field_address.span,
                                size_bits,
                                base_type: field.base_type.span.or(field.name.span),
                                field_set: parent_fs.name.span,
                            });
                            // Fix the size for now so we can continue using this field later
                            manifest.fields.get_mut(field_id).unwrap().field_address.end =
                                field.field_address.start + 64;
                            continue;
                        }
                    }
                    BaseType::FixedSize(_) => break,
                };

                manifest.fields.get_mut(field_id).unwrap().base_type.value = new_base_type;
            }
        }

        Ok(Default::default())
    }
}
