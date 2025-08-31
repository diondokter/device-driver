use anyhow::bail;

use crate::mir::Device;

use super::recurse_objects_mut;

/// Checks if the byte order is set for all registers and commands that need it and fills it out for the ones that aren't specified
pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    if let Some(default_byte_order) = device.global_config.default_byte_order {
        recurse_objects_mut(&mut device.objects, &mut |object| {
            if let Some(fs) = object.as_field_set_mut() {
                fs.byte_order = Some(default_byte_order);
            }
            Ok(())
        })?;

        return Ok(());
    }

    recurse_objects_mut(&mut device.objects, &mut |object| {
        let object_name = object.name().to_string();

        if let Some(fs) = object.as_field_set_mut() {
            if fs.size_bits > 8 && fs.byte_order.is_none() {
                bail!(
                    "No byte order is specified for fieldset `{object_name}` while it's big enough that byte order is important. Specify it on the fieldset or in the global config",
                );
            } else {
                // Even if not required, fill in a byte order so we can always unwrap it later
                fs.byte_order.get_or_insert(crate::mir::ByteOrder::LE);
            }
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use crate::mir::{ByteOrder, FieldSet, GlobalConfig, Object};

    use super::*;

    #[test]
    fn well_enough_specified() {
        let mut input = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![
                Object::FieldSet(FieldSet {
                    name: "MyRegister".into(),
                    size_bits: 8,
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "MyRegister2".into(),
                    size_bits: 9,
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
        };

        run_pass(&mut input).unwrap();
    }

    #[test]
    fn not_enough_specified() {
        let mut input = Device {
            name: None,
            global_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyRegister".into(),
                size_bits: 9,
                ..Default::default()
            })],
        };

        assert_eq!(
            run_pass(&mut input).unwrap_err().to_string(),
            "No byte order is specified for fieldset `MyRegister` while it's big enough that byte order is important. Specify it on the fieldset or in the global config"
        );
    }

    #[test]
    fn not_enough_specified_but_global_config() {
        let global_config = GlobalConfig {
            default_byte_order: Some(ByteOrder::LE),
            ..Default::default()
        };

        let mut input = Device {
            name: None,
            global_config,
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyRegister".into(),
                size_bits: 9,
                ..Default::default()
            })],
        };

        run_pass(&mut input).unwrap();
    }
}
