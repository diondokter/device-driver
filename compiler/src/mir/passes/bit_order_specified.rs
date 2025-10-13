use crate::mir::{LendingIterator, Manifest};

/// Set the unset bit orders to the device config value
pub fn run_pass(manifest: &mut Manifest) -> miette::Result<()> {
    let mut iter = manifest.iter_objects_with_config_mut();
    while let Some((object, config)) = iter.next() {
        if let Some(fs) = object.as_field_set_mut()
            && fs.bit_order.is_none()
        {
            // Set to what's in the config, or use the default LSB0
            fs.bit_order = Some(config.bit_order.unwrap_or(crate::mir::BitOrder::LSB0))
        }
    }

    Ok(())
}
