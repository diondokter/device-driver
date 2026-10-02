use std::collections::HashSet;

use device_driver_diagnostics::{Diagnostics, DynError, errors::AddressTypeUndefined};

use crate::{
    model::{Manifest, Object, ObjectId},
    passes::{Assumption, Pass},
};

/// Checks if the various address types are specified. If not an error is given out.
pub struct AddressTypesSpecified;

impl Pass for AddressTypesSpecified {
    const ASSUMPTIONS_MADE: &[Assumption] =
        &[Assumption::DeviceConfigsValid, Assumption::NamesUnique];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[Assumption::AddressTypesSpecified];

    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<ObjectId>, DynError> {
        let mut register_removals = HashSet::<ObjectId>::new();
        let mut command_removals = HashSet::<ObjectId>::new();
        let mut buffer_removals = HashSet::<ObjectId>::new();

        for (object_id, object) in manifest.objects_enumerated() {
            let config = manifest.object_config(object_id);
            match object {
                Object::Register(r) if config.register_address_type.is_none() => {
                    let device_id = config.owner.ok_or_else(|| {
                        DynError::new(
                            format!("found register {}, but the config that applies to it doesn't have an owner", r.name.original()),
                        )
                    })?;
                    if register_removals.contains(&device_id.into()) {
                        continue;
                    }

                    let device = manifest.devices.get(device_id).unwrap();

                    diagnostics.add(AddressTypeUndefined {
                        object_name: object.name_span(),
                        device: device.name.span,
                        properties_span: device.properties_span,
                        object_type: "register",
                    });
                    register_removals.insert(device_id.into());
                }
                Object::Command(c) if config.command_address_type.is_none() => {
                    let device_id = config.owner.ok_or_else(|| {
                        DynError::new(format!(
                            "found command {}, but the config that applies to it doesn't have an owner",
                            c.name.original()
                        ))
                    })?;
                    if command_removals.contains(&device_id.into()) {
                        continue;
                    }

                    let device = manifest.devices.get(device_id).unwrap();

                    diagnostics.add(AddressTypeUndefined {
                        object_name: object.name_span(),
                        device: device.name.span,
                        properties_span: device.properties_span,
                        object_type: "command",
                    });
                    command_removals.insert(device_id.into());
                }
                Object::Buffer(b) if config.buffer_address_type.is_none() => {
                    let device_id = config.owner.ok_or_else(|| {
                        DynError::new(format!(
                            "found buffer {}, but the config that applies to it doesn't have an owner",
                            b.name.original()
                        ))
                    })?;
                    if buffer_removals.contains(&device_id.into()) {
                        continue;
                    }

                    let device = manifest.devices.get(device_id).unwrap();

                    diagnostics.add(AddressTypeUndefined {
                        object_name: object.name_span(),
                        device: device.name.span,
                        properties_span: device.properties_span,
                        object_type: "buffer",
                    });
                    buffer_removals.insert(device_id.into());
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
}
