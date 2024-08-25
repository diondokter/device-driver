use anyhow::bail;

use crate::mir::{Device, Object};

use super::recurse_objects;

/// Checks if the byte order is set for all registers and commands that need it
pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    if device.global_config.default_byte_order.is_some() {
        return Ok(());
    }

    recurse_objects(&mut device.objects, &mut |object| match object {
        Object::Register(r) if r.size_bits > 8 && r.byte_order.is_none() => {
            bail!("No byte order is specified for register \"{}\" while it's big enough that byte order is important. Specify it on the register or in the global config", r.name);
        }
        Object::Command(c)
            if (c.size_bits_in > 8 || c.size_bits_out > 8) && c.byte_order.is_none() =>
        {
            bail!("No byte order is specified for command \"{}\" while it's big enough that byte order is important. Specify it on the command or in the global config", c.name);
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
