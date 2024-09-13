use anyhow::bail;

use crate::mir::{Device, Object};

use super::recurse_objects_mut;

/// Checks if the byte order is set for all registers and commands that need it and fills it out for the ones that aren't specified
pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    if let Some(default_byte_order) = device.global_config.default_byte_order {
        recurse_objects_mut(&mut device.objects, &mut |object| match object {
            Object::Register(r) if r.byte_order.is_none() => {
                r.byte_order = Some(default_byte_order);
                Ok(())
            }
            Object::Command(c) if c.byte_order.is_none() => {
                c.byte_order = Some(default_byte_order);
                Ok(())
            }
            _ => Ok(()),
        })?;

        return Ok(());
    }

    recurse_objects_mut(&mut device.objects, &mut |object| match object {
        Object::Register(r) if r.size_bits > 8 && r.byte_order.is_none() => {
            bail!("No byte order is specified for register \"{}\" while it's big enough that byte order is important. Specify it on the register or in the global config", r.name);
        }
        Object::Register(r) if r.byte_order.is_none() => {
            // Too small to matter, so just use LE
            r.byte_order = Some(crate::mir::ByteOrder::LE);
            Ok(())
        }
        Object::Command(c)
            if (c.size_bits_in > 8 || c.size_bits_out > 8) && c.byte_order.is_none() =>
        {
            bail!("No byte order is specified for command \"{}\" while it's big enough that byte order is important. Specify it on the command or in the global config", c.name);
        }
        Object::Command(c) if c.byte_order.is_none() => {
            // Too small to matter, so just use LE
            c.byte_order = Some(crate::mir::ByteOrder::LE);
            Ok(())
        }
        _ => Ok(()),
    })
}

#[cfg(test)]
mod tests {
    use crate::mir::{ByteOrder, Command, GlobalConfig, Register};

    use super::*;

    #[test]
    fn well_enough_specified() {
        let mut input = Device {
            global_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "MyRegister".into(),
                    size_bits: 8,
                    ..Default::default()
                }),
                Object::Register(Register {
                    name: "MyRegister2".into(),
                    size_bits: 9,
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
                Object::Command(Command {
                    name: "MyCommand".into(),
                    ..Default::default()
                }),
                Object::Command(Command {
                    name: "MyCommand2".into(),
                    size_bits_in: 9,
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
                Object::Command(Command {
                    name: "MyCommand3".into(),
                    size_bits_out: 9,
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
        };

        run_pass(&mut input).unwrap();
    }

    #[test]
    fn not_enough_specified() {
        let mut input = Device {
            global_config: Default::default(),
            objects: vec![Object::Register(Register {
                name: "MyRegister".into(),
                size_bits: 9,
                ..Default::default()
            })],
        };

        assert_eq!(
            run_pass(&mut input).unwrap_err().to_string(),
            "No byte order is specified for register \"MyRegister\" while it's big enough that byte order is important. Specify it on the register or in the global config"
        );

        let mut input = Device {
            global_config: Default::default(),
            objects: vec![Object::Command(Command {
                name: "MyCommand".into(),
                size_bits_in: 9,
                ..Default::default()
            })],
        };

        assert_eq!(
            run_pass(&mut input).unwrap_err().to_string(),
            "No byte order is specified for command \"MyCommand\" while it's big enough that byte order is important. Specify it on the command or in the global config"
        );

        let mut input = Device {
            global_config: Default::default(),
            objects: vec![Object::Command(Command {
                name: "MyCommand".into(),
                size_bits_out: 9,
                ..Default::default()
            })],
        };

        assert_eq!(
            run_pass(&mut input).unwrap_err().to_string(),
            "No byte order is specified for command \"MyCommand\" while it's big enough that byte order is important. Specify it on the command or in the global config"
        );
    }

    #[test]
    fn not_enough_specified_but_global_config() {
        let global_config = GlobalConfig {
            default_byte_order: Some(ByteOrder::LE),
            ..Default::default()
        };

        let mut input = Device {
            global_config,
            objects: vec![
                Object::Register(Register {
                    name: "MyRegister".into(),
                    size_bits: 9,
                    ..Default::default()
                }),
                Object::Command(Command {
                    name: "MyCommand".into(),
                    size_bits_in: 9,
                    ..Default::default()
                }),
                Object::Command(Command {
                    name: "MyCommand".into(),
                    size_bits_out: 9,
                    ..Default::default()
                }),
            ],
        };

        run_pass(&mut input).unwrap();
    }
}
