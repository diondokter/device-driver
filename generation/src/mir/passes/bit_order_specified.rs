use crate::mir::Device;

use super::recurse_objects_mut;

/// Set the unset bit orders to the global config value
pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    recurse_objects_mut(&mut device.objects, &mut |object| {
        object.field_sets_mut().for_each(|fs| {
            if fs.bit_order.is_none() {
                fs.bit_order = Some(device.global_config.default_bit_order)
            }
        });
        Ok(())
    })
}
