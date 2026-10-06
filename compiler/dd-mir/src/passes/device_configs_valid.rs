use std::collections::HashSet;

use device_driver_diagnostics::{Diagnostics, DynError};

use crate::{
    model::{Manifest, ObjectId},
    passes::{Assumption, Pass},
};

/// Sets the owner of all device configs to the actual owner of it and makes sure it's an override on the manifest config
pub struct DeviceConfigsValid;

impl Pass for DeviceConfigsValid {
    const ASSUMPTIONS_MADE: &[Assumption] = &[];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[Assumption::DeviceConfigsValid];

    fn run_pass(
        manifest: &mut Manifest,
        _diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<ObjectId>, DynError> {
        for (device_id, device) in manifest.devices.iter_enumerated_mut() {
            device.device_config = manifest.config.override_with(&device.device_config);
            device.device_config.owner = Some(device_id);
        }

        Ok(Default::default())
    }
}
