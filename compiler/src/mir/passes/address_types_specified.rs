use miette::ensure;

use crate::mir::{Device, Object};

use super::recurse_objects;

/// Checks if the various address types are specified. If not an error is given out.
pub fn run_pass(device: &mut Device) -> miette::Result<()> {
    recurse_objects(&device.objects, &mut |object| {
        match object {
            Object::Register(_) => {
                ensure!(
                    device.device_config.register_address_type.is_some(),
                    "No register address type is specified in the device config, but it's required since a register is defined."
                );
            }
            Object::Command(_) => {
                ensure!(
                    device.device_config.command_address_type.is_some(),
                    "No command address type is specified in the device config, but it's required since a command is defined."
                );
            }
            Object::Buffer(_) => {
                ensure!(
                    device.device_config.buffer_address_type.is_some(),
                    "No buffer address type is specified in the device config, but it's required since a buffer is defined."
                );
            }
            _ => {}
        }
        Ok(())
    })
}
