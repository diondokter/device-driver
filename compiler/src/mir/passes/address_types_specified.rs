use miette::ensure;

use crate::mir::{Manifest, Object};

/// Checks if the various address types are specified. If not an error is given out.
pub fn run_pass(manifest: &mut Manifest) -> miette::Result<()> {
    for (object, config) in manifest.iter_objects_with_config() {
        match object {
            Object::Register(_) => {
                ensure!(
                    config.register_address_type.is_some(),
                    "No register address type is specified in the device config, but it's required since a register is defined."
                );
            }
            Object::Command(_) => {
                ensure!(
                    config.command_address_type.is_some(),
                    "No command address type is specified in the device config, but it's required since a command is defined."
                );
            }
            Object::Buffer(_) => {
                ensure!(
                    config.buffer_address_type.is_some(),
                    "No buffer address type is specified in the device config, but it's required since a buffer is defined."
                );
            }
            _ => {}
        }
    }

    Ok(())
}
