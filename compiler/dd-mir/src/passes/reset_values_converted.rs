use std::collections::HashMap;

use bitvec::{
    order::{Lsb0, Msb0},
    view::BitView,
};
use device_driver_common::{
    span::{SpanExt, Spanned},
    specifiers::{BitOrder, ByteOrder, ResetValue},
};

use crate::{
    model::{FieldSet, LendingIterator, Manifest, Object, Register, Unique},
    search_object,
};
use device_driver_diagnostics::{
    Diagnostics,
    errors::{ResetValueArrayWrongSize, ResetValueTooBig},
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
                    target_field_set
                        .bit_order
                        .expect("Bitorder should be set at this point"),
                    target_field_set.size_bits.value,
                    target_field_set.byte_order.unwrap(),
                    diagnostics,
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
    bit_order: BitOrder,
    size_bits: u32,
    target_byte_order: ByteOrder,
    diagnostics: &mut Diagnostics,
) -> Option<Spanned<ResetValue>> {
    let target_byte_size = size_bits.div_ceil(8) as usize;

    match reset_value.value {
        ResetValue::Integer(int) => {
            // Convert the integer to LE and LSB0
            let mut array = int.to_le_bytes();
            if bit_order == BitOrder::MSB0 {
                array.iter_mut().for_each(|b| *b = b.reverse_bits());
            }

            let array_view = array.view_bits_mut::<Lsb0>();

            // Check if the value is not too big
            if array_view[size_bits as usize..].any() {
                diagnostics.add_miette(ResetValueTooBig {
                    reset_value: reset_value.span,
                    reset_value_size_bits: (array_view.len() - array_view.trailing_zeros()) as u32,
                    register_size_bits: size_bits,
                });
                return None;
            }

            let mut final_array = array[..target_byte_size].to_vec();

            // Flip the bitorder back to the target, but crucially keep the byte order little endian!
            if bit_order == BitOrder::MSB0 {
                final_array.iter_mut().for_each(|b| *b = b.reverse_bits());
            }

            // Convert to big endian if required. Bitvec's output is always little endian
            if target_byte_order == ByteOrder::BE {
                final_array.reverse();
            }

            Some(ResetValue::Array(final_array).with_span(reset_value.span))
        }
        ResetValue::Array(mut array) => {
            if array.len() != target_byte_size {
                diagnostics.add_miette(ResetValueArrayWrongSize {
                    reset_value: reset_value.span,
                    reset_value_size_bytes: array.len() as u32,
                    register_size_bytes: target_byte_size as u32,
                });
                return None;
            }

            // Convert to little endian to do the check since that's what bitvec needs
            if target_byte_order == ByteOrder::BE {
                array.reverse();
            }

            match bit_order {
                BitOrder::LSB0 => {
                    if array.view_bits::<Lsb0>()[size_bits as usize..].any() {
                        diagnostics.add_miette(ResetValueTooBig {
                            reset_value: reset_value.span,
                            reset_value_size_bits: (array.view_bits::<Lsb0>().len()
                                - array.view_bits::<Lsb0>().trailing_zeros())
                                as u32,
                            register_size_bits: size_bits,
                        });
                        return None;
                    }
                }
                BitOrder::MSB0 => {
                    if array.view_bits::<Msb0>()[size_bits as usize..].any() {
                        diagnostics.add_miette(ResetValueTooBig {
                            reset_value: reset_value.span,
                            reset_value_size_bits: (array.view_bits::<Msb0>().len()
                                - array.view_bits::<Msb0>().trailing_zeros())
                                as u32,
                            register_size_bits: size_bits,
                        });
                        return None;
                    }
                }
            }

            // Convert back to big endian
            if target_byte_order == ByteOrder::BE {
                array.reverse();
            }

            Some(ResetValue::Array(array).with_span(reset_value.span))
        }
    }
}

#[cfg(test)]
mod tests {
    use device_driver_common::{identifier::IdentifierRef, span::Span};

    use crate::model::{Device, DeviceConfig, FieldSet, Register};

    use super::*;

    #[test]
    fn correct_sizes() {
        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into_with_dummy_span(),
                    reset_value: Some(ResetValue::Integer(0x1F).with_dummy_span()),
                    field_set_ref: IdentifierRef::new("fs".into()),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into_with_dummy_span(),
                    size_bits: 5.with_dummy_span(),
                    bit_order: Some(BitOrder::LSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
            span: Span::default(),
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
        assert!(!diagnostics.has_error());

        let end_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into_with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0x1F]).with_dummy_span()),
                    field_set_ref: IdentifierRef::new("fs".into()),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into_with_dummy_span(),
                    size_bits: 5.with_dummy_span(),
                    bit_order: Some(BitOrder::LSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
            span: Span::default(),
        }
        .into();

        assert_eq!(start_mir, end_mir);

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into_with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0x1F]).with_dummy_span()),
                    field_set_ref: IdentifierRef::new("fs".into()),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into_with_dummy_span(),
                    size_bits: 5.with_dummy_span(),
                    bit_order: Some(BitOrder::LSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
            span: Span::default(),
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
        assert!(!diagnostics.has_error());

        let end_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into_with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0x1F]).with_dummy_span()),
                    field_set_ref: IdentifierRef::new("fs".into()),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into_with_dummy_span(),
                    size_bits: 5.with_dummy_span(),
                    bit_order: Some(BitOrder::LSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
            span: Span::default(),
        }
        .into();

        assert_eq!(start_mir, end_mir);

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: DeviceConfig {
                byte_order: Some(ByteOrder::LE),
                ..Default::default()
            },
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into_with_dummy_span(),
                    reset_value: Some(ResetValue::Integer(0x423).with_dummy_span()),
                    field_set_ref: IdentifierRef::new("fs".into()),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into_with_dummy_span(),
                    size_bits: 11.with_dummy_span(),
                    bit_order: Some(BitOrder::LSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
            span: Span::default(),
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
        assert!(!diagnostics.has_error());

        let end_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: DeviceConfig {
                byte_order: Some(ByteOrder::LE),
                ..Default::default()
            },
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into_with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0x23, 0x04]).with_dummy_span()),
                    field_set_ref: IdentifierRef::new("fs".into()),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into_with_dummy_span(),
                    size_bits: 11.with_dummy_span(),
                    bit_order: Some(BitOrder::LSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
            span: Span::default(),
        }
        .into();

        assert_eq!(start_mir, end_mir);

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into_with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0x04, 0x23]).with_dummy_span()),
                    field_set_ref: IdentifierRef::new("fs".into()),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into_with_dummy_span(),
                    size_bits: 11.with_dummy_span(),
                    byte_order: Some(ByteOrder::BE),
                    bit_order: Some(BitOrder::LSB0),
                    ..Default::default()
                }),
            ],
            span: Span::default(),
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
        assert!(!diagnostics.has_error());

        let end_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into_with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0x04, 0x23]).with_dummy_span()),
                    field_set_ref: IdentifierRef::new("fs".into()),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into_with_dummy_span(),
                    size_bits: 11.with_dummy_span(),
                    byte_order: Some(ByteOrder::BE),
                    bit_order: Some(BitOrder::LSB0),
                    ..Default::default()
                }),
            ],
            span: Span::default(),
        }
        .into();

        assert_eq!(start_mir, end_mir);

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into_with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0x20, 0xC4]).with_dummy_span()),
                    field_set_ref: IdentifierRef::new("fs".into()),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into_with_dummy_span(),
                    size_bits: 11.with_dummy_span(),
                    byte_order: Some(ByteOrder::BE),
                    bit_order: Some(BitOrder::MSB0),
                    ..Default::default()
                }),
            ],
            span: Span::default(),
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
        assert!(!diagnostics.has_error());

        let end_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into_with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0x20, 0xC4]).with_dummy_span()),
                    field_set_ref: IdentifierRef::new("fs".into()),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into_with_dummy_span(),
                    size_bits: 11.with_dummy_span(),
                    byte_order: Some(ByteOrder::BE),
                    bit_order: Some(BitOrder::MSB0),
                    ..Default::default()
                }),
            ],
            span: Span::default(),
        }
        .into();

        assert_eq!(start_mir, end_mir);
    }

    #[test]
    fn incorrect_sizes() {
        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: DeviceConfig {
                byte_order: Some(ByteOrder::LE),
                ..Default::default()
            },
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into_with_dummy_span(),
                    reset_value: Some(ResetValue::Integer(0x423).with_dummy_span()),
                    field_set_ref: IdentifierRef::new("fs".into()),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into_with_dummy_span(),
                    size_bits: 10.with_dummy_span(),
                    bit_order: Some(BitOrder::LSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
            span: Span::default(),
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
        assert!(diagnostics.has_error());

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into_with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0x04, 0x23]).with_dummy_span()),
                    field_set_ref: IdentifierRef::new("fs".into()),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into_with_dummy_span(),
                    size_bits: 10.with_dummy_span(),
                    byte_order: Some(ByteOrder::BE),
                    bit_order: Some(BitOrder::LSB0),
                    ..Default::default()
                }),
            ],
            span: Span::default(),
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
        assert!(diagnostics.has_error());

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into_with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0x20, 0xC4]).with_dummy_span()),
                    field_set_ref: IdentifierRef::new("fs".into()),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into_with_dummy_span(),
                    size_bits: 10.with_dummy_span(),
                    byte_order: Some(ByteOrder::BE),
                    bit_order: Some(BitOrder::MSB0),
                    ..Default::default()
                }),
            ],
            span: Span::default(),
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
        assert!(diagnostics.has_error());
    }

    #[test]
    fn wrong_num_bytes_array() {
        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into_with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0, 0, 0]).with_dummy_span()),
                    field_set_ref: IdentifierRef::new("fs".into()),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into_with_dummy_span(),
                    size_bits: 32.with_dummy_span(),
                    byte_order: Some(ByteOrder::LE),
                    bit_order: Some(BitOrder::LSB0),
                    ..Default::default()
                }),
            ],
            span: Span::default(),
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
        assert!(diagnostics.has_error());
    }

    #[test]
    fn int_msb0_parsed_correct() {
        let mut start_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into_with_dummy_span(),
                    reset_value: Some(ResetValue::Integer(0xF8).with_dummy_span()),
                    field_set_ref: IdentifierRef::new("fs".into()),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into_with_dummy_span(),
                    size_bits: 5.with_dummy_span(),
                    bit_order: Some(BitOrder::MSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
            span: Span::default(),
        }
        .into();

        let mut diagnostics = Diagnostics::new();
        run_pass(&mut start_mir, &mut diagnostics);
        assert!(!diagnostics.has_error());

        let end_mir = Device {
            description: String::new(),
            name: "Device".into_with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into_with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0xF8]).with_dummy_span()),
                    field_set_ref: IdentifierRef::new("fs".into()),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into_with_dummy_span(),
                    size_bits: 5.with_dummy_span(),
                    bit_order: Some(BitOrder::MSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
            span: Span::default(),
        }
        .into();

        assert_eq!(start_mir, end_mir);
    }
}
