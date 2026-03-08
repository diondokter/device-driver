use crate::model::{LendingIterator, Manifest, Object, Unique};

/// Sets the owner of all device configs to the actual owner of it
pub fn run_pass(manifest: &mut Manifest) {
    let mut iter = manifest.iter_objects_with_config_mut();
    while let Some((object, _)) = iter.next() {
        let Object::Device(device) = object else {
            continue;
        };

        device.device_config.owner = Some(device.id());
    }
}
