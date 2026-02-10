use crate::model::{LendingIterator, Manifest};
use device_driver_common::specifiers::BaseType;
use device_driver_diagnostics::{Diagnostics, errors::BoolFieldTooLarge};

/// Check all bool fields. They must be exactly zero or one bits
pub fn run_pass(manifest: &mut Manifest, diagnostics: &mut Diagnostics) {
    let mut iter = manifest.iter_objects_with_config_mut();
    while let Some((object, _)) = iter.next() {
        let Some(field_set) = object.as_field_set_mut() else {
            continue;
        };

        for field in field_set.fields.iter_mut() {
            if field.base_type == BaseType::Bool {
                // When zero bits long, extend to one bit
                if field.field_address.start == field.field_address.end {
                    field.field_address.end += 1;
                }

                if field.field_address.value.clone().count() != 1 {
                    diagnostics.add(BoolFieldTooLarge {
                        base_type: if field.base_type.span.is_empty() {
                            None
                        } else {
                            Some(field.base_type.span)
                        },
                        address: field.field_address.span,
                        address_bits: field.field_address.len() as u32,
                        address_start: field.field_address.start,

                        field_set_context: field_set.name.span,
                    });
                    // To fix for further use, set the len to just 1
                    field.field_address.end = field.field_address.start + 1;
                }
            }
        }
    }
}
