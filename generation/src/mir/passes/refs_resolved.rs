use crate::mir::{Device, Object};

use super::recurse_objects_mut;

/// Turn every ref into a concrete type.
/// This assumes all refs are valid, so run [super::refs_validated] first.
pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {


    recurse_objects_mut(&mut device.objects, &mut |object| {
        match object {
            crate::mir::Object::Ref(ref_object) => {
                todo!()
            }
            _ => {}
        };

        Ok(())
    })
}

fn search_object<'d>(name: &str, objects: &'d [Object]) -> Option<&'d Object> {
    for object in objects {
        if object.name() == name {
            return Some(object);
        }

        if let Some(block_objects) = object.get_block_object_list() {
            match search_object(name, block_objects) {
                None => {}
                found => return found,
            }
        }
    }

    None
}
