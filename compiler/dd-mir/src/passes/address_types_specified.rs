use std::collections::HashSet;

use device_driver_diagnostics::{Diagnostics, DynError, errors::AddressTypeUndefined};

use crate::{
    model::{Manifest, Object, UniqueId},
    search_object,
};

/// Checks if the various address types are specified. If not an error is given out.
pub fn run_pass(
    manifest: &mut Manifest,
    diagnostics: &mut Diagnostics,
) -> Result<HashSet<UniqueId>, DynError> {
    let mut register_removals = HashSet::new();
    let mut command_removals = HashSet::new();
    let mut buffer_removals = HashSet::new();

    for (object, config) in manifest.iter_objects_with_config() {
        match object {
            Object::Register(r) if config.register_address_type.is_none() => {
                let device = config.owner.as_ref().ok_or_else(|| {
                    DynError::new(
                        format!("found register {}, but the config that applies to it doesn't have an owner", r.name.original()),
                    )
                })?;
                if register_removals.contains(device) {
                    continue;
                }

                let device_first_object = search_object(manifest, &device.identifier().take_ref())
                    .ok_or_else(|| DynError::new("config owner doesn't exist for register"))?
                    .child_objects()
                    .first()
                    .ok_or_else(|| {
                        DynError::new("device that owns register doesn't have children")
                    })?
                    .span();

                diagnostics.add(AddressTypeUndefined {
                    object_name: object.name_span(),
                    device: device.span(),
                    device_config_area: device_first_object,
                    object_type: "register",
                });
                register_removals.insert(device.clone());
            }
            Object::Command(c) if config.command_address_type.is_none() => {
                let device = config.owner.as_ref().ok_or_else(|| {
                    DynError::new(format!(
                        "found command {}, but the config that applies to it doesn't have an owner",
                        c.name.original()
                    ))
                })?;
                if command_removals.contains(device) {
                    continue;
                }

                let device_first_object = search_object(manifest, &device.identifier().take_ref())
                    .ok_or_else(|| DynError::new("config owner doesn't exist for command"))?
                    .child_objects()
                    .first()
                    .ok_or_else(|| DynError::new("device that owns command doesn't have children"))?
                    .span();

                diagnostics.add(AddressTypeUndefined {
                    object_name: object.name_span(),
                    device: device.span(),
                    device_config_area: device_first_object,
                    object_type: "command",
                });
                command_removals.insert(device.clone());
            }
            Object::Buffer(b) if config.buffer_address_type.is_none() => {
                let device = config.owner.as_ref().ok_or_else(|| {
                    DynError::new(format!(
                        "found buffer {}, but the config that applies to it doesn't have an owner",
                        b.name.original()
                    ))
                })?;
                if buffer_removals.contains(device) {
                    continue;
                }

                let device_first_object = search_object(manifest, &device.identifier().take_ref())
                    .ok_or_else(|| DynError::new("config owner doesn't exist for buffer"))?
                    .child_objects()
                    .first()
                    .ok_or_else(|| DynError::new("device that owns buffer doesn't have children"))?
                    .span();

                diagnostics.add(AddressTypeUndefined {
                    object_name: object.name_span(),
                    device: device.span(),
                    device_config_area: device_first_object,
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
    Ok(removals)
}
