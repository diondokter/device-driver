use std::collections::HashMap;

use bitvec::{
    order::{Lsb0, Msb0},
    view::BitView,
};
use miette::ensure;

use crate::mir::{
    BitOrder, ByteOrder, FieldSet, LendingIterator, Manifest, Object, Register, ResetValue, Unique,
    passes::search_object,
};

/// Checks if the reset values of registers are valid.
/// Also converts integer values to the array representation using the correct bit and byte order.
///
/// For the array representation, the rule is that the input must have the same spec as the bit and byte order.
/// The reset values are left with the specified bit order and byte order.
///
/// This function assumes all register have a valid byte order, and so depends on [super::byte_order_specified::run_pass]
/// having been run.
pub fn run_pass(manifest: &mut Manifest) -> miette::Result<()> {
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
                    target_field_set.size_bits,
                    "register",
                    &reg.name,
                    target_field_set.byte_order.unwrap(),
                )?;
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
            register.reset_value = Some(new_reset_value);
        }
    }

    assert!(new_reset_values.is_empty());

    Ok(())
}

fn get_target_field_set<'m>(reg: &Register, manifest: &'m Manifest) -> &'m FieldSet {
    search_object(manifest, &reg.field_set_ref.0)
        .expect("All fieldset refs should already be checked and valid here")
        .as_field_set()
        .expect("All fieldset refs should already be checked and valid here")
}

fn convert_reset_value(
    reset_value: ResetValue,
    bit_order: BitOrder,
    size_bits: u32,
    object_type_name: &str,
    object_name: &str,
    target_byte_order: ByteOrder,
) -> miette::Result<ResetValue> {
    let target_byte_size = size_bits.div_ceil(8) as usize;

    match reset_value {
        ResetValue::Integer(int) => {
            // Convert the integer to LE and LSB0
            let mut array = int.to_le_bytes();
            if bit_order == BitOrder::MSB0 {
                array.iter_mut().for_each(|b| *b = b.reverse_bits());
            }

            let array_view = array.view_bits_mut::<Lsb0>();

            // Check if the value is not too big
            ensure!(
                !array_view[size_bits as usize..].any(),
                "The reset value of {object_type_name} `{object_name}` has (a) bit(s) specified above the size of the register. \
                While you can specify them, this is likely a mistake and thus not accepted. Keep the bits `{size_bits}..` all at zero",
            );

            let mut final_array = array[..target_byte_size].to_vec();

            // Flip the bitorder back to the target, but crucially keep the byte order little endian!
            if bit_order == BitOrder::MSB0 {
                final_array.iter_mut().for_each(|b| *b = b.reverse_bits());
            }

            // Convert to big endian if required. Bitvec's output is always little endian
            if target_byte_order == ByteOrder::BE {
                final_array.reverse();
            }

            Ok(ResetValue::Array(final_array))
        }
        ResetValue::Array(mut array) => {
            ensure!(
                array.len() == target_byte_size,
                "The reset value of {object_type_name} `{}` has the incorrect length. It must be specified as {target_byte_size} bytes, but now only has {} elements",
                object_name,
                array.len(),
            );

            // Convert to little endian to do the check since that's what bitvec needs
            if target_byte_order == ByteOrder::BE {
                array.reverse();
            }

            match bit_order {
                BitOrder::LSB0 => {
                    ensure!(
                        !array.view_bits::<Lsb0>()[size_bits as usize..].any(),
                        "The reset value of {object_type_name} `{object_name}` has (a) bit(s) specified above the size of the register. \
                        While you can specify them, this is likely a mistake and thus not accepted. Keep the bits `{size_bits}..` all at zero",
                    );
                }
                BitOrder::MSB0 => {
                    ensure!(
                        !array.view_bits::<Msb0>()[size_bits as usize..].any(),
                        "The reset value of {object_type_name} `{object_name}` has (a) bit(s) specified above the size of the register. \
                        While you can specify them, this is likely a mistake and thus not accepted. Keep the bits `{size_bits}..` all at zero",
                    );
                }
            }

            // Convert back to big endian
            if target_byte_order == ByteOrder::BE {
                array.reverse();
            }

            Ok(ResetValue::Array(array))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mir::{Device, DeviceConfig, FieldSet, Register, Span};

    use super::*;

    #[test]
    fn correct_sizes() {
        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".to_owned().with_dummy_span(),
                    reset_value: Some(ResetValue::Integer(0x1F)),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".to_owned().with_dummy_span(),
                    size_bits: 5,
                    bit_order: Some(BitOrder::LSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
        }
        .into();

        run_pass(&mut start_mir).unwrap();

        let end_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".to_owned().with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0x1F])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".to_owned().with_dummy_span(),
                    size_bits: 5,
                    bit_order: Some(BitOrder::LSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
        }
        .into();

        assert_eq!(start_mir, end_mir);

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".to_owned().with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0x1F])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".to_owned().with_dummy_span(),
                    size_bits: 5,
                    bit_order: Some(BitOrder::LSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
        }
        .into();

        run_pass(&mut start_mir).unwrap();

        let end_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".to_owned().with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0x1F])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".to_owned().with_dummy_span(),
                    size_bits: 5,
                    bit_order: Some(BitOrder::LSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
        }
        .into();

        assert_eq!(start_mir, end_mir);

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: DeviceConfig {
                byte_order: Some(ByteOrder::LE),
                ..Default::default()
            },
            objects: vec![
                Object::Register(Register {
                    name: "Reg".to_owned().with_dummy_span(),
                    reset_value: Some(ResetValue::Integer(0x423)),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".to_owned().with_dummy_span(),
                    size_bits: 11,
                    bit_order: Some(BitOrder::LSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
        }
        .into();

        run_pass(&mut start_mir).unwrap();

        let end_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: DeviceConfig {
                byte_order: Some(ByteOrder::LE),
                ..Default::default()
            },
            objects: vec![
                Object::Register(Register {
                    name: "Reg".to_owned().with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0x23, 0x04])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".to_owned().with_dummy_span(),
                    size_bits: 11,
                    bit_order: Some(BitOrder::LSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
        }
        .into();

        assert_eq!(start_mir, end_mir);

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".to_owned().with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0x04, 0x23])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".to_owned().with_dummy_span(),
                    size_bits: 11,
                    byte_order: Some(ByteOrder::BE),
                    bit_order: Some(BitOrder::LSB0),
                    ..Default::default()
                }),
            ],
        }
        .into();

        run_pass(&mut start_mir).unwrap();

        let end_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".to_owned().with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0x04, 0x23])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".to_owned().with_dummy_span(),
                    size_bits: 11,
                    byte_order: Some(ByteOrder::BE),
                    bit_order: Some(BitOrder::LSB0),
                    ..Default::default()
                }),
            ],
        }
        .into();

        assert_eq!(start_mir, end_mir);

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".to_owned().with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0x20, 0xC4])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".to_owned().with_dummy_span(),
                    size_bits: 11,
                    byte_order: Some(ByteOrder::BE),
                    bit_order: Some(BitOrder::MSB0),
                    ..Default::default()
                }),
            ],
        }
        .into();

        run_pass(&mut start_mir).unwrap();

        let end_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".to_owned().with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0x20, 0xC4])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".to_owned().with_dummy_span(),
                    size_bits: 11,
                    byte_order: Some(ByteOrder::BE),
                    bit_order: Some(BitOrder::MSB0),
                    ..Default::default()
                }),
            ],
        }
        .into();

        assert_eq!(start_mir, end_mir);
    }

    #[test]
    fn incorrect_sizes() {
        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: DeviceConfig {
                byte_order: Some(ByteOrder::LE),
                ..Default::default()
            },
            objects: vec![
                Object::Register(Register {
                    name: "Reg".to_owned().with_dummy_span(),
                    reset_value: Some(ResetValue::Integer(0x423)),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".to_owned().with_dummy_span(),
                    size_bits: 10,
                    bit_order: Some(BitOrder::LSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
        }
        .into();

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "The reset value of register `Reg` has (a) bit(s) specified above the size of the register. While you can specify them, this is likely a mistake and thus not accepted. Keep the bits `10..` all at zero"
        );

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".to_owned().with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0x04, 0x23])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".to_owned().with_dummy_span(),
                    size_bits: 10,
                    byte_order: Some(ByteOrder::BE),
                    bit_order: Some(BitOrder::LSB0),
                    ..Default::default()
                }),
            ],
        }
        .into();

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "The reset value of register `Reg` has (a) bit(s) specified above the size of the register. While you can specify them, this is likely a mistake and thus not accepted. Keep the bits `10..` all at zero"
        );

        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".to_owned().with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0x20, 0xC4])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".to_owned().with_dummy_span(),
                    size_bits: 10,
                    byte_order: Some(ByteOrder::BE),
                    bit_order: Some(BitOrder::MSB0),
                    ..Default::default()
                }),
            ],
        }
        .into();

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "The reset value of register `Reg` has (a) bit(s) specified above the size of the register. While you can specify them, this is likely a mistake and thus not accepted. Keep the bits `10..` all at zero"
        );
    }

    #[test]
    fn wrong_num_bytes_array() {
        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".to_owned().with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0, 0, 0])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".to_owned().with_dummy_span(),
                    size_bits: 32,
                    byte_order: Some(ByteOrder::LE),
                    bit_order: Some(BitOrder::LSB0),
                    ..Default::default()
                }),
            ],
        }
        .into();

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "The reset value of register `Reg` has the incorrect length. It must be specified as 4 bytes, but now only has 3 elements"
        );
    }

    #[test]
    fn int_msb0_parsed_correct() {
        let mut start_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".to_owned().with_dummy_span(),
                    reset_value: Some(ResetValue::Integer(0xF8)),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".to_owned().with_dummy_span(),
                    size_bits: 5,
                    bit_order: Some(BitOrder::MSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
        }
        .into();

        run_pass(&mut start_mir).unwrap();

        let end_mir = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".to_owned().with_dummy_span(),
                    reset_value: Some(ResetValue::Array(vec![0xF8])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".to_owned().with_dummy_span(),
                    size_bits: 5,
                    bit_order: Some(BitOrder::MSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
        }
        .into();

        assert_eq!(start_mir, end_mir);
    }
}
