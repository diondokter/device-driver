use std::collections::{HashMap, HashSet};

use anyhow::ensure;

use crate::mir::{Device, Object, ObjectOverride};

use super::recurse_objects;

/// Checks if all refs are valid
pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    let mut reffed_blocks = HashMap::new();
    let mut reffed_registers = HashMap::new();
    let mut reffed_commands = HashMap::new();

    let mut real_blocks = HashSet::new();
    let mut real_registers = HashSet::new();
    let mut real_commands = HashSet::new();

    recurse_objects(&device.objects, &mut |object| {
        match object {
            Object::Ref(r) => {
                match r.object_override {
                    ObjectOverride::Block(_) => {
                        reffed_blocks.insert(r.object_override.name().to_string(), r.name.clone())
                    }
                    ObjectOverride::Register(_) => reffed_registers
                        .insert(r.object_override.name().to_string(), r.name.clone()),
                    ObjectOverride::Command(_) => {
                        reffed_commands.insert(r.object_override.name().to_string(), r.name.clone())
                    }
                };
            }
            Object::Block(v) => {
                real_blocks.insert(v.name.clone());
            }
            Object::Register(v) => {
                real_registers.insert(v.name.clone());
            }
            Object::Command(v) => {
                real_commands.insert(v.name.clone());
            }
            Object::Buffer(_) => {}
        }

        Ok(())
    })?;

    for (ref_target_name, reffer_name) in reffed_blocks {
        ensure!(
            real_blocks.contains(&ref_target_name),
            "Block ref `{reffer_name}` refers to unknown block `{ref_target_name}`"
        );
    }

    for (ref_target_name, reffer_name) in reffed_registers {
        ensure!(
            real_registers.contains(&ref_target_name),
            "Register ref `{reffer_name}` refers to unknown register `{ref_target_name}`"
        );
    }

    for (ref_target_name, reffer_name) in reffed_commands {
        ensure!(
            real_commands.contains(&ref_target_name),
            "Command ref `{reffer_name}` refers to unknown command `{ref_target_name}`"
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::mir::{
        BlockOverride, CommandOverride, ObjectOverride, RefObject, Register, RegisterOverride,
    };

    use super::*;

    #[test]
    fn ref_found() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![
                Object::Register(Register {
                    name: "MyReg".into(),
                    ..Default::default()
                }),
                Object::Ref(RefObject {
                    cfg_attr: Default::default(),
                    description: Default::default(),
                    name: "MyRef".into(),
                    object_override: ObjectOverride::Register(RegisterOverride {
                        name: "MyReg".into(),
                        ..Default::default()
                    }),
                }),
            ],
        };

        run_pass(&mut start_mir).unwrap();
    }

    #[test]
    fn bad_register_ref() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Ref(RefObject {
                cfg_attr: Default::default(),
                description: Default::default(),
                name: "MyRef".into(),
                object_override: ObjectOverride::Register(RegisterOverride {
                    name: "MyReg2".into(),
                    ..Default::default()
                }),
            })],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Register ref `MyRef` refers to unknown register `MyReg2`"
        );
    }

    #[test]
    fn bad_block_ref() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Ref(RefObject {
                cfg_attr: Default::default(),
                description: Default::default(),
                name: "MyRef".into(),
                object_override: ObjectOverride::Block(BlockOverride {
                    name: "MyBlock2".into(),
                    ..Default::default()
                }),
            })],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Block ref `MyRef` refers to unknown block `MyBlock2`"
        );
    }

    #[test]
    fn bad_command_ref() {
        let mut start_mir = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::Ref(RefObject {
                cfg_attr: Default::default(),
                description: Default::default(),
                name: "MyRef".into(),
                object_override: ObjectOverride::Command(CommandOverride {
                    name: "MyComm2".into(),
                    ..Default::default()
                }),
            })],
        };

        assert_eq!(
            run_pass(&mut start_mir).unwrap_err().to_string(),
            "Command ref `MyRef` refers to unknown command `MyComm2`"
        );
    }
}
