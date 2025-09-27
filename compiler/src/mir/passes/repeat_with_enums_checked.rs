use miette::bail;

use crate::mir::{Device, Object, RepeatSource, passes::search_object};

use super::recurse_objects;

/// Checks if the enums referenced by repeats actually exist
pub fn run_pass(device: &mut Device) -> miette::Result<()> {
    recurse_objects(&device.objects, &mut |object| {
        let object_name = object.name();
        let object_type = object.type_name();

        if let Some(repeat) = object.repeat() {
            if let RepeatSource::Enum(enum_name) = &repeat.source {
                match search_object(&device.objects, &enum_name) {
                    Some(Object::Enum(_)) => {}
                    _ => {
                        bail!(
                            "Cannot find the enum called `{enum_name}` that's used in the repeat specified in {object_type} `{object_name}`"
                        )
                    }
                }
            }
        }

        Ok(())
    })
}
