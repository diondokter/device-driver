use std::collections::HashSet;

use device_driver_diagnostics::{Diagnostics, DynError};

use crate::{
    model::{LendingIterator, Manifest, Object, Unique, UniqueId},
    passes::{Assumption, Pass},
};

/// Sets the owner of all device configs to the actual owner of it
pub struct DeviceConfigsOwned;

impl Pass for DeviceConfigsOwned {
    const ASSUMPTIONS_MADE: &[Assumption] = &[];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[Assumption::DeviceConfigsOwned];

    fn run_pass(
        manifest: &mut Manifest,
        _diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<UniqueId>, DynError> {
        let mut iter = manifest.iter_objects_with_config_mut();
        while let Some((object, _)) = iter.next() {
            let Object::Device(device) = object else {
                continue;
            };

            device.device_config.owner = Some(device.id());
        }

        Ok(Default::default())
    }
}
