use crate::mir::{Cfg, Device, FieldConversion};

use super::recurse_objects_with_depth_mut;

/// Propagate the cfg attributes. This makes sure that any child will be able to be active when the parent is not.
///
/// Currently this is done by `all(...)` combining the cfgs. In the future this should have a more advanced check.
pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    let mut current_depth = 0;
    let mut cfg_stack = vec![Cfg::new(None)];

    recurse_objects_with_depth_mut(&mut device.objects, &mut |object, depth| {
        if depth < current_depth {
            cfg_stack.pop();
            current_depth = depth;
        }

        let cfg_attr = object.cfg_attr_mut();
        let new_cfg_attr = cfg_attr.combine(cfg_stack.last().unwrap());
        *cfg_attr = new_cfg_attr.clone();

        for field in object.field_sets_mut().flatten() {
            // NOTE: We don't have to set the field cfg attr since it's part of the object
            // We do need to update the enum though, since that's gonna be generated outside of the object
            if let Some(FieldConversion::Enum { enum_value, .. }) = field.field_conversion.as_mut()
            {
                enum_value.cfg_attr = field.cfg_attr.combine(&new_cfg_attr);
                // Just like we don't have to update the field cfg, we also don't have to update the enum variant cfgs
            }
        }

        if object.as_block_mut().is_some() {
            cfg_stack.push(new_cfg_attr);
            current_depth += 1;
        }

        Ok(())
    })
}
