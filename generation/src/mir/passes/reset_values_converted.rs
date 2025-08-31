use std::collections::HashMap;

use anyhow::ensure;
use bitvec::{
    order::{Lsb0, Msb0},
    view::BitView,
};

use crate::mir::{BitOrder, ByteOrder, Device, FieldSet, Object, Register, ResetValue, Unique};

use super::{recurse_objects, recurse_objects_mut};

/// Checks if the reset values of registers are valid.
/// Also converts integer values to the array representation using the correct bit and byte order.
///
/// For the array representation, the rule is that the input must have the same spec as the bit and byte order.
/// The reset values are left with the specified bit order and byte order.
///
/// This function assumes all register have a valid byte order, and so depends on [super::byte_order_specified::run_pass]
/// having been run.
pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    let mut new_reset_values = HashMap::new();

    recurse_objects(&device.objects, &mut |object| match object {
        Object::Register(reg) => {
            let target_field_set = get_target_field_set(reg, device);

            match reg.reset_value.as_ref() {
                Some(reset_value) => {
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
                    Ok(())
                }
                None => Ok(()),
            }
        }
        _ => Ok(()),
    })?;

    recurse_objects_mut(&mut device.objects, &mut |object| match object {
        Object::Register(register) => {
            if let Some(new_reset_value) = new_reset_values.remove(&register.id()) {
                register.reset_value = Some(new_reset_value);
            }

            Ok(())
        }
        _ => Ok(()),
    })?;

    assert!(new_reset_values.is_empty());

    Ok(())
}

fn get_target_field_set(reg: &Register, device: &Device) -> FieldSet {
    let mut field_set = None;

    recurse_objects(&device.objects, &mut |object| {
        if let Object::FieldSet(fs) = object
            && fs.name == reg.field_set_ref.0
        {
            field_set = Some(fs.clone());
        }

        Ok(())
    })
    .unwrap();

    field_set.expect("All fieldset refs should already be checked and valid here")
}

fn convert_reset_value(
    reset_value: ResetValue,
    bit_order: BitOrder,
    size_bits: u32,
    object_type_name: &str,
    object_name: &str,
    target_byte_order: ByteOrder,
) -> anyhow::Result<ResetValue> {
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
                "The reset value of {object_type_name} `{}` has (a) bit(s) specified above the size of the register. \
                While you can specify them, this is likely a mistake and thus not accepted. Keep the bits `{}..` all at zero",
                object_name,
                size_bits,
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
                        "The reset value of {object_type_name} `{}` has (a) bit(s) specified above the size of the register. \
                        While you can specify them, this is likely a mistake and thus not accepted. Keep the bits `{}..` all at zero",
                        object_name,
                        size_bits,
                    );
                }
                BitOrder::MSB0 => {
                    ensure!(
                        !array.view_bits::<Msb0>()[size_bits as usize..].any(),
                        "The reset value of {object_type_name} `{}` has (a) bit(s) specified above the size of the register. \
                        While you can specify them, this is likely a mistake and thus not accepted. Keep the bits `{}..` all at zero",
                        object_name,
                        size_bits,
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
    use crate::mir::{FieldSet, GlobalConfig, Register};

    use super::*;

    #[test]
    fn correct_sizes() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into(),
                    reset_value: Some(ResetValue::Integer(0x1F)),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into(),
                    size_bits: 5,
                    bit_order: Some(BitOrder::LSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
        };

        run_pass(&mut start_mir).unwrap();

        let end_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into(),
                    reset_value: Some(ResetValue::Array(vec![0x1F])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into(),
                    size_bits: 5,
                    bit_order: Some(BitOrder::LSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
        };

        assert_eq!(start_mir, end_mir);

        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into(),
                    reset_value: Some(ResetValue::Array(vec![0x1F])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into(),
                    size_bits: 5,
                    bit_order: Some(BitOrder::LSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
        };

        run_pass(&mut start_mir).unwrap();

        let end_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into(),
                    reset_value: Some(ResetValue::Array(vec![0x1F])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into(),
                    size_bits: 5,
                    bit_order: Some(BitOrder::LSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
        };

        assert_eq!(start_mir, end_mir);

        let mut start_mir = Device {
            name: None,
            global_config: GlobalConfig {
                default_byte_order: Some(ByteOrder::LE),
                ..Default::default()
            },
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into(),
                    reset_value: Some(ResetValue::Integer(0x423)),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into(),
                    size_bits: 11,
                    bit_order: Some(BitOrder::LSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
        };

        run_pass(&mut start_mir).unwrap();

        let end_mir = Device {
            name: None,
            global_config: GlobalConfig {
                default_byte_order: Some(ByteOrder::LE),
                ..Default::default()
            },
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into(),
                    reset_value: Some(ResetValue::Array(vec![0x23, 0x04])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into(),
                    size_bits: 11,
                    bit_order: Some(BitOrder::LSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
        };

        assert_eq!(start_mir, end_mir);

        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into(),
                    reset_value: Some(ResetValue::Array(vec![0x04, 0x23])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into(),
                    size_bits: 11,
                    byte_order: Some(ByteOrder::BE),
                    bit_order: Some(BitOrder::LSB0),
                    ..Default::default()
                }),
            ],
        };

        run_pass(&mut start_mir).unwrap();

        let end_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into(),
                    reset_value: Some(ResetValue::Array(vec![0x04, 0x23])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into(),
                    size_bits: 11,
                    byte_order: Some(ByteOrder::BE),
                    bit_order: Some(BitOrder::LSB0),
                    ..Default::default()
                }),
            ],
        };

        assert_eq!(start_mir, end_mir);

        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into(),
                    reset_value: Some(ResetValue::Array(vec![0x20, 0xC4])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into(),
                    size_bits: 11,
                    byte_order: Some(ByteOrder::BE),
                    bit_order: Some(BitOrder::MSB0),
                    ..Default::default()
                }),
            ],
        };

        run_pass(&mut start_mir).unwrap();

        let end_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into(),
                    reset_value: Some(ResetValue::Array(vec![0x20, 0xC4])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into(),
                    size_bits: 11,
                    byte_order: Some(ByteOrder::BE),
                    bit_order: Some(BitOrder::MSB0),
                    ..Default::default()
                }),
            ],
        };

        assert_eq!(start_mir, end_mir);
    }

    #[test]
    fn incorrect_sizes() {
        let mut start_mir = Device {
            name: None,
            global_config: GlobalConfig {
                default_byte_order: Some(ByteOrder::LE),
                ..Default::default()
            },
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into(),
                    reset_value: Some(ResetValue::Integer(0x423)),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into(),
                    size_bits: 10,
                    bit_order: Some(BitOrder::LSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "The reset value of register `Reg` has (a) bit(s) specified above the size of the register. While you can specify them, this is likely a mistake and thus not accepted. Keep the bits `10..` all at zero"
        );

        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into(),
                    reset_value: Some(ResetValue::Array(vec![0x04, 0x23])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into(),
                    size_bits: 10,
                    byte_order: Some(ByteOrder::BE),
                    bit_order: Some(BitOrder::LSB0),
                    ..Default::default()
                }),
            ],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "The reset value of register `Reg` has (a) bit(s) specified above the size of the register. While you can specify them, this is likely a mistake and thus not accepted. Keep the bits `10..` all at zero"
        );

        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into(),
                    reset_value: Some(ResetValue::Array(vec![0x20, 0xC4])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into(),
                    size_bits: 10,
                    byte_order: Some(ByteOrder::BE),
                    bit_order: Some(BitOrder::MSB0),
                    ..Default::default()
                }),
            ],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "The reset value of register `Reg` has (a) bit(s) specified above the size of the register. While you can specify them, this is likely a mistake and thus not accepted. Keep the bits `10..` all at zero"
        );
    }

    #[test]
    fn wrong_num_bytes_arry() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into(),
                    reset_value: Some(ResetValue::Array(vec![0, 0, 0])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into(),
                    size_bits: 32,
                    byte_order: Some(ByteOrder::LE),
                    bit_order: Some(BitOrder::LSB0),
                    ..Default::default()
                }),
            ],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "The reset value of register `Reg` has the incorrect length. It must be specified as 4 bytes, but now only has 3 elements"
        );
    }

    #[test]
    fn int_msb0_parsed_correct() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into(),
                    reset_value: Some(ResetValue::Integer(0xF8)),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into(),
                    size_bits: 5,
                    bit_order: Some(BitOrder::MSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
        };

        run_pass(&mut start_mir).unwrap();

        let end_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "Reg".into(),
                    reset_value: Some(ResetValue::Array(vec![0xF8])),
                    field_set_ref: "fs".into(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "fs".into(),
                    size_bits: 5,
                    bit_order: Some(BitOrder::MSB0),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
        };

        assert_eq!(start_mir, end_mir);
    }
}
