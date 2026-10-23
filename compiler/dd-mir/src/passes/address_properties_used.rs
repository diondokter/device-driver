use std::collections::HashSet;

use crate::{
    model::{Manifest, ObjectId, ObjectType},
    passes::{Assumption, Pass},
};
use device_driver_diagnostics::{Diagnostics, DynError, errors::UnusedAddressProperty};

/// Checks if all address properties are used
pub struct AddressPropertiesUsed;

impl Pass for AddressPropertiesUsed {
    const ASSUMPTIONS_MADE: &[Assumption] = &[Assumption::DeviceConfigsValid];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[];

    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<ObjectId>, DynError> {
        let mut manifest_has_used_register_type = false;
        let mut manifest_has_used_command_type = false;
        let mut manifest_has_used_buffer_type = false;
        let mut manifest_has_used_register_mode = false;

        for device in manifest.devices.iter() {
            let mut has_register_child = false;
            let mut has_command_child = false;
            let mut has_buffer_child = false;

            let mut children = device.children.clone();

            while let Some(child) = children.pop() {
                match child.object_type() {
                    ObjectType::Register => {
                        has_register_child = true;
                    }
                    ObjectType::Command => {
                        has_command_child = true;
                    }
                    ObjectType::Buffer => {
                        has_buffer_child = true;
                    }
                    _ => {
                        if let Some(grand_children) = child.get(manifest).unwrap().child_objects() {
                            children.extend(grand_children);
                        }
                    }
                }
            }

            if let Some(register_address_type) = device.device_config.register_address_type {
                if device.span.overlaps(register_address_type.span) {
                    // This register address type is defined on the device
                    if !has_register_child {
                        diagnostics.add(UnusedAddressProperty {
                            property: register_address_type.span,
                            manifest: false,
                            property_type: "register address type",
                            child_type: "register",
                        });
                    }
                } else {
                    // Defined in manifest
                    manifest_has_used_register_type |= has_register_child;
                }
            }
            if let Some(command_address_type) = device.device_config.command_address_type {
                if device.span.overlaps(command_address_type.span) {
                    // This command address type is defined on the device
                    if !has_command_child {
                        diagnostics.add(UnusedAddressProperty {
                            property: command_address_type.span,
                            manifest: false,
                            property_type: "command address type",
                            child_type: "command",
                        });
                    }
                } else {
                    // Defined in manifest
                    manifest_has_used_command_type |= has_command_child;
                }
            }
            if let Some(buffer_address_type) = device.device_config.buffer_address_type {
                if device.span.overlaps(buffer_address_type.span) {
                    // This buffer address type is defined on the device
                    if !has_buffer_child {
                        diagnostics.add(UnusedAddressProperty {
                            property: buffer_address_type.span,
                            manifest: false,
                            property_type: "buffer address type",
                            child_type: "buffer",
                        });
                    }
                } else {
                    // Defined in manifest
                    manifest_has_used_buffer_type |= has_buffer_child;
                }
            }
            if let Some(register_address_mode) = device.device_config.register_address_mode {
                if device.span.overlaps(register_address_mode.span) {
                    // This register address mode is defined on the device
                    if !has_register_child {
                        diagnostics.add(UnusedAddressProperty {
                            property: register_address_mode.span,
                            manifest: false,
                            property_type: "register address mode",
                            child_type: "register",
                        });
                    }
                } else {
                    // Defined in manifest
                    manifest_has_used_register_mode |= has_register_child;
                }
            }
        }

        if let Some(register_address_type) = manifest.config.register_address_type
            && !manifest_has_used_register_type
        {
            diagnostics.add(UnusedAddressProperty {
                property: register_address_type.span,
                manifest: true,
                property_type: "register address type",
                child_type: "register",
            });
        }
        if let Some(command_address_type) = manifest.config.command_address_type
            && !manifest_has_used_command_type
        {
            diagnostics.add(UnusedAddressProperty {
                property: command_address_type.span,
                manifest: true,
                property_type: "command address type",
                child_type: "command",
            });
        }
        if let Some(buffer_address_type) = manifest.config.buffer_address_type
            && !manifest_has_used_buffer_type
        {
            diagnostics.add(UnusedAddressProperty {
                property: buffer_address_type.span,
                manifest: true,
                property_type: "buffer address type",
                child_type: "buffer",
            });
        }
        if let Some(register_address_mode) = manifest.config.register_address_mode
            && !manifest_has_used_register_mode
        {
            diagnostics.add(UnusedAddressProperty {
                property: register_address_mode.span,
                manifest: true,
                property_type: "register address mode",
                child_type: "register",
            });
        }

        Ok(Default::default())
    }
}
