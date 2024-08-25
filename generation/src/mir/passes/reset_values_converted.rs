use anyhow::ensure;
use bitvec::{
    order::{Lsb0, Msb0},
    view::BitView,
};

use crate::mir::{BitOrder, ByteOrder, Device, Object, ResetValue};

use super::recurse_objects;

/// Checks if the reset values of registers is valid.
/// Also converts integer values to the array representation using the correct bit and byte order.
///
/// For the array representation, the rule is that the input must have the same spec as the bit and byte order.
/// The reset values are left with the specified bit order, but always in the little endian byte order to match
/// the behaviour of bitvec.
///
/// This function assumes all register have a valid byte order, and so depends on [super::byte_order_specified::run_pass]
/// having been run.
pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    recurse_objects(&mut device.objects, &mut |object| match object {
        Object::Register(reg) => {
            let target_byte_order = reg
                .byte_order
                .or(device.global_config.default_byte_order)
                .or((reg.size_bits <= 8).then_some(ByteOrder::LE))
                .expect("Register should have a valid byte order or not need one");

            let target_byte_size = reg.size_bits.div_ceil(8) as usize;

            match reg.reset_value.as_mut() {
                Some(ResetValue::Integer(int)) => {
                    // Convert the integer to LE and LSB0
                    let mut array = int.to_le_bytes();
                    if reg.bit_order == BitOrder::MSB0 {
                        array.iter_mut().for_each(|b| *b = b.reverse_bits());
                    }

                    let array_view = array.view_bits_mut::<Lsb0>();

                    // Check if the value is not too big
                    ensure!(
                        !array_view[reg.size_bits as usize..].any(),
                        "The reset value of register \"{}\" has (a) bit(s) specified above the size of the register. \
                        While you can specify them, this is likely a mistake and thus not accepted. Keep the bits `{}..` all at zero",
                        reg.name,
                        reg.size_bits,
                    );

                    let mut final_array = array[..target_byte_size].to_vec();

                    // Flip the bitorder back to the target, but crucially keep the byte order little endian!
                    if reg.bit_order == BitOrder::MSB0 {
                        final_array.iter_mut().for_each(|b| *b = b.reverse_bits());
                    }

                    reg.reset_value = Some(ResetValue::Array(final_array));

                    Ok(())
                }
                Some(ResetValue::Array(array)) => {
                    ensure!(
                        array.len() == target_byte_size,
                        "The reset value of register \"{}\" has the incorrect length. It must be specified as {target_byte_size} bytes, but now only has {} elements",
                        reg.name,
                        array.len(),
                    );

                    // Convert to little endian since that's the output we need
                    if target_byte_order == ByteOrder::BE {
                        array.reverse();
                    }

                    match reg.bit_order {
                        BitOrder::LSB0 => {
                            ensure!(
                                !array.view_bits::<Lsb0>()[reg.size_bits as usize..].any(),
                                "The reset value of register \"{}\" has (a) bit(s) specified above the size of the register. \
                                While you can specify them, this is likely a mistake and thus not accepted. Keep the bits `{}..` all at zero",
                                reg.name,
                                reg.size_bits,
                            );
                        }
                        BitOrder::MSB0 => {
                            ensure!(
                                !array.view_bits::<Msb0>()[reg.size_bits as usize..].any(),
                                "The reset value of register \"{}\" has (a) bit(s) specified above the size of the register. \
                                While you can specify them, this is likely a mistake and thus not accepted. Keep the bits `{}..` all at zero",
                                reg.name,
                                reg.size_bits,
                            );
                        }
                    }

                    Ok(())
                }
                None => Ok(()),
            }
        }
        _ => Ok(()),
    })
}

#[cfg(test)]
mod tests {
    use crate::mir::{GlobalConfig, Register};

    use super::*;

    #[test]
    fn correct_sizes() {
        let mut start_mir = Device {
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
            global_config: Default::default(),
            objects: vec![Object::Register(Register {
                name: "Reg".into(),
                size_bits: 11,
                byte_order: Some(ByteOrder::BE),
                reset_value: Some(ResetValue::Array(vec![0x23, 0x04])),
                ..Default::default()
            })],
        };

        assert_eq!(start_mir, end_mir);

        let mut start_mir = Device {
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
            global_config: Default::default(),
            objects: vec![Object::Register(Register {
                name: "Reg".into(),
                size_bits: 11,
                byte_order: Some(ByteOrder::BE),
                bit_order: BitOrder::MSB0,
                reset_value: Some(ResetValue::Array(vec![0xC4, 0x20])),
                ..Default::default()
            })],
        };

        assert_eq!(start_mir, end_mir);
    }

    #[test]
    fn incorrect_sizes() {
        let mut start_mir = Device {
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
            "The reset value of register \"Reg\" has (a) bit(s) specified above the size of the register. While you can specify them, this is likely a mistake and thus not accepted. Keep the bits `10..` all at zero"
        );

        let mut start_mir = Device {
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
            "The reset value of register \"Reg\" has (a) bit(s) specified above the size of the register. While you can specify them, this is likely a mistake and thus not accepted. Keep the bits `10..` all at zero"
        );

        let mut start_mir = Device {
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
            "The reset value of register \"Reg\" has (a) bit(s) specified above the size of the register. While you can specify them, this is likely a mistake and thus not accepted. Keep the bits `10..` all at zero"
        );
    }

    #[test]
    fn wrong_num_bytes_arry() {
        let mut start_mir = Device {
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
            "The reset value of register \"Reg\" has the incorrect length. It must be specified as 4 bytes, but now only has 3 elements"
        );
    }

    #[test]
    fn int_msb0_parsed_correct() {
        let mut start_mir = Device {
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
