use std::collections::HashMap;

use anyhow::ensure;
use bitvec::{
    order::{Lsb0, Msb0},
    view::BitView,
};

use crate::mir::{
    BitOrder, ByteOrder, Device, Object, ObjectOverride, RefObject, Register, ResetValue, Unique,
};

use super::{recurse_objects, recurse_objects_mut, search_object};

/// Checks if the reset values of registers (and ref registers) are valid.
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
            let target_byte_order = get_target_byte_order(reg, device);

            match reg.reset_value.as_ref() {
                Some(reset_value) => {
                    let new_reset_value = convert_reset_value(
                        reset_value.clone(),
                        reg.bit_order,
                        reg.size_bits,
                        "register",
                        &reg.name,
                        target_byte_order,
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
        Object::Ref(
            ref_object @ RefObject {
                name,
                object_override: ObjectOverride::Register(reg_override),
                ..
            },
        ) => match reg_override.reset_value.as_ref() {
            Some(reset_value) => {
                let base_reg = search_object(&reg_override.name, &device.objects)
                    .expect("Refs have been validated already for existance")
                    .as_register()
                    .expect("Refs have been validated already for types");

                let target_byte_order = get_target_byte_order(base_reg, device);

                let new_reset_value = convert_reset_value(
                    reset_value.clone(),
                    base_reg.bit_order,
                    base_reg.size_bits,
                    "ref register",
                    name,
                    target_byte_order,
                )?;
                new_reset_values.insert(ref_object.id(), new_reset_value);
                Ok(())
            }
            None => Ok(()),
        },
        _ => Ok(()),
    })?;

    recurse_objects_mut(&mut device.objects, &mut |object| match object {
        Object::Register(register) => {
            if let Some(new_reset_value) = new_reset_values.remove(&register.id()) {
                register.reset_value = Some(new_reset_value);
            }

            Ok(())
        }
        Object::Ref(
            ref_object @ RefObject {
                object_override: ObjectOverride::Register(_),
                ..
            },
        ) => {
            if let Some(new_reset_value) = new_reset_values.remove(&ref_object.id()) {
                ref_object
                    .object_override
                    .as_register_mut()
                    .unwrap()
                    .reset_value = Some(new_reset_value);
            }
            Ok(())
        }
        _ => Ok(()),
    })?;

    assert!(new_reset_values.is_empty());

    Ok(())
}

fn get_target_byte_order(reg: &Register, device: &Device) -> ByteOrder {
    reg.byte_order
        .or(device.global_config.default_byte_order)
        .or((reg.size_bits <= 8).then_some(ByteOrder::LE))
        .expect("Register should have a valid byte order or not need one")
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
    use crate::mir::{GlobalConfig, Register};

    use super::*;

    #[test]
    fn correct_sizes() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Register(Register {
                name: "Reg".into(),
                size_bits: 5,
                reset_value: Some(ResetValue::Integer(0x1F)),
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        let end_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Register(Register {
                name: "Reg".into(),
                size_bits: 5,
                reset_value: Some(ResetValue::Array(vec![0x1F])),
                ..Default::default()
            })],
        };

        assert_eq!(start_mir, end_mir);

        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Register(Register {
                name: "Reg".into(),
                size_bits: 5,
                reset_value: Some(ResetValue::Array(vec![0x1F])),
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        let end_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Register(Register {
                name: "Reg".into(),
                size_bits: 5,
                reset_value: Some(ResetValue::Array(vec![0x1F])),
                ..Default::default()
            })],
        };

        assert_eq!(start_mir, end_mir);

        let mut start_mir = Device {
            name: None,
            global_config: GlobalConfig {
                default_byte_order: Some(ByteOrder::LE),
                ..Default::default()
            },
            objects: vec![Object::Register(Register {
                name: "Reg".into(),
                size_bits: 11,
                reset_value: Some(ResetValue::Integer(0x423)),
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        let end_mir = Device {
            name: None,
            global_config: GlobalConfig {
                default_byte_order: Some(ByteOrder::LE),
                ..Default::default()
            },
            objects: vec![Object::Register(Register {
                name: "Reg".into(),
                size_bits: 11,
                reset_value: Some(ResetValue::Array(vec![0x23, 0x04])),
                ..Default::default()
            })],
        };

        assert_eq!(start_mir, end_mir);

        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Register(Register {
                name: "Reg".into(),
                size_bits: 11,
                byte_order: Some(ByteOrder::BE),
                reset_value: Some(ResetValue::Array(vec![0x04, 0x23])),
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        let end_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Register(Register {
                name: "Reg".into(),
                size_bits: 11,
                byte_order: Some(ByteOrder::BE),
                reset_value: Some(ResetValue::Array(vec![0x04, 0x23])),
                ..Default::default()
            })],
        };

        assert_eq!(start_mir, end_mir);

        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Register(Register {
                name: "Reg".into(),
                size_bits: 11,
                byte_order: Some(ByteOrder::BE),
                bit_order: BitOrder::MSB0,
                reset_value: Some(ResetValue::Array(vec![0x20, 0xC4])),
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        let end_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Register(Register {
                name: "Reg".into(),
                size_bits: 11,
                byte_order: Some(ByteOrder::BE),
                bit_order: BitOrder::MSB0,
                reset_value: Some(ResetValue::Array(vec![0x20, 0xC4])),
                ..Default::default()
            })],
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
            objects: vec![Object::Register(Register {
                name: "Reg".into(),
                size_bits: 10,
                reset_value: Some(ResetValue::Integer(0x423)),
                ..Default::default()
            })],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "The reset value of register `Reg` has (a) bit(s) specified above the size of the register. While you can specify them, this is likely a mistake and thus not accepted. Keep the bits `10..` all at zero"
        );

        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Register(Register {
                name: "Reg".into(),
                size_bits: 10,
                byte_order: Some(ByteOrder::BE),
                reset_value: Some(ResetValue::Array(vec![0x04, 0x23])),
                ..Default::default()
            })],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "The reset value of register `Reg` has (a) bit(s) specified above the size of the register. While you can specify them, this is likely a mistake and thus not accepted. Keep the bits `10..` all at zero"
        );

        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Register(Register {
                name: "Reg".into(),
                size_bits: 10,
                byte_order: Some(ByteOrder::BE),
                bit_order: BitOrder::MSB0,
                reset_value: Some(ResetValue::Array(vec![0x20, 0xC4])),
                ..Default::default()
            })],
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
            objects: vec![Object::Register(Register {
                name: "Reg".into(),
                size_bits: 32,
                byte_order: Some(ByteOrder::LE),
                reset_value: Some(ResetValue::Array(vec![0, 0, 0])),
                ..Default::default()
            })],
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
            objects: vec![Object::Register(Register {
                name: "Reg".into(),
                size_bits: 5,
                bit_order: BitOrder::MSB0,
                reset_value: Some(ResetValue::Integer(0xF8)),
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();

        let end_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Register(Register {
                name: "Reg".into(),
                size_bits: 5,
                bit_order: BitOrder::MSB0,
                reset_value: Some(ResetValue::Array(vec![0xF8])),
                ..Default::default()
            })],
        };

        assert_eq!(start_mir, end_mir);
    }
}
