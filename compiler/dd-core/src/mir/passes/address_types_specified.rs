use std::collections::HashSet;

use crate::{
    mir::{Manifest, Object, UniqueId},
    reporting::{Diagnostics, errors::AddressTypeUndefined},
};

/// Checks if the various address types are specified. If not an error is given out.
pub fn run_pass(manifest: &mut Manifest, diagnostics: &mut Diagnostics) -> HashSet<UniqueId> {
    let mut register_removals = HashSet::new();
    let mut command_removals = HashSet::new();
    let mut buffer_removals = HashSet::new();

    for (object, config) in manifest.iter_objects_with_config() {
        match object {
            Object::Register(_) if config.register_address_type.is_none() => {
                let device = config.owner.as_ref().expect(
                    "Registers are always defined in a device, thus the config has an owner",
                );
                if register_removals.contains(device) {
                    continue;
                }

                diagnostics.add(AddressTypeUndefined {
                    object: object.name_span(),
                    config_device: device.span(),
                    object_type: "register",
                });
                register_removals.insert(device.clone());
            }
            Object::Command(_) if config.command_address_type.is_none() => {
                let device = config.owner.as_ref().expect(
                    "Commands are always defined in a device, thus the config has an owner",
                );
                if command_removals.contains(device) {
                    continue;
                }

                diagnostics.add(AddressTypeUndefined {
                    object: object.name_span(),
                    config_device: device.span(),
                    object_type: "command",
                });
                command_removals.insert(device.clone());
            }
            Object::Buffer(_) if config.buffer_address_type.is_none() => {
                let device = config
                    .owner
                    .as_ref()
                    .expect("Buffers are always defined in a device, thus the config has an owner");
                if buffer_removals.contains(device) {
                    continue;
                }

                diagnostics.add(AddressTypeUndefined {
                    object: object.name_span(),
                    config_device: device.span(),
                    object_type: "buffer",
                });
                buffer_removals.insert(device.clone());
            }
            _ => {}
        }
    }

    let mut removals = HashSet::new();
    removals.extend(register_removals);
    removals.extend(command_removals);
    removals.extend(buffer_removals);
    removals
}
