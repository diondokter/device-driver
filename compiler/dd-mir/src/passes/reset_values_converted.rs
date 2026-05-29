use std::collections::HashMap;

use device_driver_common::{
    span::{Span, SpanExt, Spanned},
    specifiers::{ByteOrder, ResetValue},
};

use crate::{
    model::{FieldSet, LendingIterator, Manifest, Object, Register, Unique},
    search_object,
};
use device_driver_diagnostics::{
    Diagnostics,
    errors::{ResetValueArrayWrongSize, ResetValueIntTooBig},
};

/// Checks if the reset values of registers are valid.
/// Also converts integer values to the array representation using the correct bit and byte order.
///
/// For the array representation, the rule is that the input must have the same spec as the bit and byte order.
/// The reset values are left with the specified bit order and byte order.
///
/// This function assumes all register have a valid byte order, and so depends on [`super::byte_order_specified::run_pass`]
/// having been run.
pub fn run_pass(manifest: &mut Manifest, diagnostics: &mut Diagnostics) {
    let mut new_reset_values = HashMap::new();

    for object in manifest.iter_objects() {
        if let Object::Register(reg) = object {
            let target_field_set = get_target_field_set(reg, manifest);

            if let Some(reset_value) = reg.reset_value.as_ref() {
                let new_reset_value = convert_reset_value(
                    reset_value.clone(),
                    target_field_set.size_bytes.value,
                    target_field_set.byte_order.unwrap(),
                    diagnostics,
                    reg.name.span,
                );
                assert_eq!(
                    new_reset_values.insert(reg.id(), new_reset_value),
                    None,
                    "All names must be unique"
                );
            }
        }
    }

    let mut iter = manifest.iter_objects_with_config_mut();
    while let Some((object, _)) = iter.next() {
        if let Object::Register(register) = object
            && let Some(new_reset_value) = new_reset_values.remove(&register.id())
        {
            register.reset_value = new_reset_value;
        }
    }

    assert!(new_reset_values.is_empty());
}

fn get_target_field_set<'m>(reg: &Register, manifest: &'m Manifest) -> &'m FieldSet {
    search_object(manifest, &reg.field_set_ref)
        .expect("All fieldset refs should already be checked and valid here")
        .as_field_set()
        .expect("All fieldset refs should already be checked and valid here")
}

fn convert_reset_value(
    reset_value: Spanned<ResetValue>,
    size_bytes: u32,
    target_byte_order: ByteOrder,
    diagnostics: &mut Diagnostics,
    register_context: Span,
) -> Option<Spanned<ResetValue>> {
    match reset_value.value {
        ResetValue::Integer(int) => match target_byte_order {
            ByteOrder::LE => {
                let int_bytes = int.to_le_bytes();

                let (val, rest) = int_bytes
                    .split_at_checked(size_bytes as usize)
                    .unwrap_or((&int_bytes, &[]));

                if rest.iter().any(|b| *b != 0) {
                    diagnostics.add(ResetValueIntTooBig {
                        reset_value: reset_value.span,
                        reset_value_size_bytes: size_bytes
                            + (rest.len() - rest.iter().rev().take_while(|b| **b == 0).count())
                                as u32,
                        register_size_bytes: size_bytes,
                        register_context,
                    });
                    return None;
                }

                let mut val = val.to_vec();

                if val.len() < size_bytes as usize {
                    // Add 0's behind
                    val.resize(size_bytes as usize, 0);
                }

                Some(ResetValue::Array(val).with_span(reset_value.span))
            }
            ByteOrder::BE => {
                let int_bytes = int.to_be_bytes();
                let (rest, val) =
                    int_bytes.split_at(int_bytes.len().saturating_sub(size_bytes as usize));

                if rest.iter().any(|b| *b != 0) {
                    diagnostics.add(ResetValueIntTooBig {
                        reset_value: reset_value.span,
                        reset_value_size_bytes: size_bytes
                            + (rest.len() - rest.iter().take_while(|b| **b == 0).count()) as u32,
                        register_size_bytes: size_bytes,
                        register_context,
                    });
                    return None;
                }

                let mut val = val.to_vec();

                if val.len() < size_bytes as usize {
                    // Add 0's in front
                    let mut new_val = vec![0; size_bytes as usize];
                    new_val[size_bytes as usize - val.len()..].copy_from_slice(&val);
                    val = new_val;
                }

                Some(ResetValue::Array(val).with_span(reset_value.span))
            }
        },
        ResetValue::Array(array) => {
            if array.len() != size_bytes as usize {
                diagnostics.add(ResetValueArrayWrongSize {
                    reset_value: reset_value.span,
                    reset_value_size_bytes: array.len() as u32,
                    register_size_bytes: size_bytes,
                    register_context,
                });
                return None;
            }

            Some(ResetValue::Array(array).with_span(reset_value.span))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_pass() {
        // LE
        let mut diagnostics = Diagnostics::new();
        assert_eq!(
            convert_reset_value(
                ResetValue::Integer(0x12_34).with_dummy_span(),
                2,
                ByteOrder::LE,
                &mut diagnostics,
                Span::empty()
            ),
            Some(ResetValue::Array(vec![0x34, 0x12]).with_dummy_span())
        );
        assert!(diagnostics.is_empty());

        // BE
        let mut diagnostics = Diagnostics::new();
        assert_eq!(
            convert_reset_value(
                ResetValue::Integer(0x12_34).with_dummy_span(),
                2,
                ByteOrder::BE,
                &mut diagnostics,
                Span::empty()
            ),
            Some(ResetValue::Array(vec![0x12, 0x34]).with_dummy_span())
        );
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn integer_fail() {
        // LE
        let mut diagnostics = Diagnostics::new();
        assert_eq!(
            convert_reset_value(
                ResetValue::Integer(0x12_34_56).with_dummy_span(),
                1,
                ByteOrder::LE,
                &mut diagnostics,
                Span::empty()
            ),
            None
        );
        assert!(diagnostics.has_error());
        println!("{diagnostics:?}");

        // BE
        let mut diagnostics = Diagnostics::new();
        assert_eq!(
            convert_reset_value(
                ResetValue::Integer(0x12_34_56).with_dummy_span(),
                1,
                ByteOrder::BE,
                &mut diagnostics,
                Span::empty()
            ),
            None
        );
        assert!(diagnostics.has_error());
        println!("{diagnostics:?}");
    }

    #[test]
    fn array_pass() {
        let mut diagnostics = Diagnostics::new();
        assert_eq!(
            convert_reset_value(
                ResetValue::Array(vec![0x12, 0x34]).with_dummy_span(),
                2,
                ByteOrder::LE,
                &mut diagnostics,
                Span::empty()
            ),
            Some(ResetValue::Array(vec![0x12, 0x34]).with_dummy_span())
        );
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn array_fail() {
        let mut diagnostics = Diagnostics::new();
        assert_eq!(
            convert_reset_value(
                ResetValue::Array(vec![0x12, 0x34, 0x56]).with_dummy_span(),
                2,
                ByteOrder::LE,
                &mut diagnostics,
                Span::empty()
            ),
            None
        );
        assert!(diagnostics.has_error());
        println!("{diagnostics:?}");

        let mut diagnostics = Diagnostics::new();
        assert_eq!(
            convert_reset_value(
                ResetValue::Array(vec![0x12]).with_dummy_span(),
                2,
                ByteOrder::LE,
                &mut diagnostics,
                Span::empty()
            ),
            None
        );
        assert!(diagnostics.has_error());
        println!("{diagnostics:?}");
    }
}
