use miette::ensure;

use crate::mir::{Device, Object};

use super::find_min_max_addresses;

/// Checks if the various address types can fully contain the min and max addresses of the types of objects they are for
pub fn run_pass(device: &mut Device) -> miette::Result<()> {
    if let Some(register_address_type) = device.global_config.register_address_type {
        let (min_address, max_address) = find_min_max_addresses(&device.objects, |o| {
            matches!(o, Object::Block(_) | Object::Register(_))
        });

        ensure!(
            min_address >= register_address_type.min_value(),
            "The register addresses go as low as {min_address}, but the selected address type `{register_address_type}` only goes down to {}. Choose an address type that can fit the full address range",
            register_address_type.min_value()
        );
        ensure!(
            max_address <= register_address_type.max_value(),
            "The register addresses go as high as {max_address}, but the selected address type `{register_address_type}` only goes up to {}. Choose an address type that can fit the full address range",
            register_address_type.max_value()
        );
    }

    if let Some(command_address_type) = device.global_config.command_address_type {
        let (min_address, max_address) = find_min_max_addresses(&device.objects, |o| {
            matches!(o, Object::Block(_) | Object::Command(_))
        });

        ensure!(
            min_address >= command_address_type.min_value(),
            "The command addresses go as low as {min_address}, but the selected address type `{command_address_type}` only goes down to {}. Choose an address type that can fit the full address range",
            command_address_type.min_value()
        );
        ensure!(
            max_address <= command_address_type.max_value(),
            "The command addresses go as high as {max_address}, but the selected address type `{command_address_type}` only goes up to {}. Choose an address type that can fit the full address range",
            command_address_type.max_value()
        );
    }

    if let Some(buffer_address_type) = device.global_config.buffer_address_type {
        let (min_address, max_address) = find_min_max_addresses(&device.objects, |o| {
            matches!(o, Object::Block(_) | Object::Buffer(_))
        });

        ensure!(
            min_address >= buffer_address_type.min_value(),
            "The buffer addresses go as low as {min_address}, but the selected address type `{buffer_address_type}` only goes down to {}. Choose an address type that can fit the full address range",
            buffer_address_type.min_value()
        );
        ensure!(
            max_address <= buffer_address_type.max_value(),
            "The buffer addresses go as high as {max_address}, but the selected address type `{buffer_address_type}` only goes up to {}. Choose an address type that can fit the full address range",
            buffer_address_type.max_value()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::mir::{Command, GlobalConfig, Integer, Register};

    use super::*;

    #[test]
    fn not_too_low() {
        let mut start_mir = Device {
            name: None,
            global_config: GlobalConfig {
                register_address_type: Some(Integer::I8),
                ..Default::default()
            },
            objects: vec![Object::Register(Register {
                name: "MyReg".into(),
                address: -300,
                ..Default::default()
            })],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "The register addresses go as low as -300, but the selected address type `i8` only goes down to -128. Choose an address type that can fit the full address range"
        );
    }

    #[test]
    fn not_too_high() {
        let mut start_mir = Device {
            name: None,
            global_config: GlobalConfig {
                command_address_type: Some(Integer::U16),
                ..Default::default()
            },
            objects: vec![Object::Command(Command {
                name: "MyReg".into(),
                address: 128000,
                ..Default::default()
            })],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "The command addresses go as high as 128000, but the selected address type `u16` only goes up to 65535. Choose an address type that can fit the full address range"
        );
    }
}
