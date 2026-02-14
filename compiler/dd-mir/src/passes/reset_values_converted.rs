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
                    target_field_set.size_bits.value,
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
    size_bits: u32,
    target_byte_order: ByteOrder,
    diagnostics: &mut Diagnostics,
    register_context: Span,
) -> Option<Spanned<ResetValue>> {
    let target_byte_size = size_bits.div_ceil(8) as usize;

    match reset_value.value {
        ResetValue::Integer(int) => {
            // Assert int is a u128. The rest of the calculations bank on that
            let _: u128 = int;

            let used_bits = int.checked_ilog2().map(|log| log + 1).unwrap_or_default();

            if used_bits > size_bits {
                diagnostics.add(ResetValueTooBig {
                    reset_value: reset_value.span,
                    reset_value_size_bits: used_bits,
                    register_size_bits: size_bits,
                    register_context,
                });
                return None;
            }

            let mut array = vec![0; target_byte_size];
            match target_byte_order {
                ByteOrder::LE => {
                    let num_bytes_used = target_byte_size.min(8);
                    array[..num_bytes_used].copy_from_slice(&int.to_le_bytes()[..num_bytes_used]);
                }
                ByteOrder::BE => {
                    let tmp = int.to_be_bytes();
                    let tmp_slice = &tmp[tmp.len() - target_byte_size.min(8)..];
                    array[target_byte_size - tmp_slice.len()..].copy_from_slice(tmp_slice);
                }
            };

            convert_reset_value(
                ResetValue::Array(array).with_span(reset_value.span),
                size_bits,
                target_byte_order,
                diagnostics,
                register_context,
            )
        }
        ResetValue::Array(array) => {
            if array.len() != target_byte_size {
                diagnostics.add(ResetValueArrayWrongSize {
                    reset_value: reset_value.span,
                    reset_value_size_bytes: array.len() as u32,
                    register_size_bytes: target_byte_size as u32,
                    register_context,
                });
                return None;
            }

            if array.is_empty() {
                return Some(ResetValue::Array(array).with_span(reset_value.span));
            }

            let biggest_byte = match target_byte_order {
                ByteOrder::LE => array.last().unwrap(),
                ByteOrder::BE => array.first().unwrap(),
            };

            let used_bits = biggest_byte
                .checked_ilog2()
                .map(|log| log + 1)
                .unwrap_or_default()
                + (target_byte_size as u32 - 1) * 8;

            // diagnostics.add(Message::new(format!("bit_order: {bit_order}, lsb0_biggest_byte: {lsb0_biggest_byte:#010b}, target_byte_size: {target_byte_size}, used bits: {used_bits}")));

            // Check if the value is not too big
            if used_bits > size_bits {
                diagnostics.add(ResetValueTooBig {
                    reset_value: reset_value.span,
                    reset_value_size_bits: used_bits,
                    register_size_bits: size_bits,
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
}
