use miette::bail;

use crate::mir::{LendingIterator, Manifest};

/// Checks if the byte order is set for all registers and commands that need it and fills it out for the ones that aren't specified
pub fn run_pass(manifest: &mut Manifest) -> miette::Result<()> {
    let mut iter = manifest.iter_objects_with_config_mut();
    while let Some((object, config)) = iter.next() {
        if let Some(fs) = object.as_field_set_mut() {
            if fs.byte_order.is_none() {
                fs.byte_order = config.byte_order;
            }

            if fs.size_bits > 8 && fs.byte_order.is_none() {
                bail!(
                    "No byte order is specified for fieldset `{}` while it's big enough that byte order is important. Specify it on the fieldset or in the device config",
                    object.name()
                );
            } else {
                // Even if not required, fill in a byte order so we can always unwrap it later
                fs.byte_order.get_or_insert(crate::mir::ByteOrder::LE);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::mir::{ByteOrder, Device, DeviceConfig, FieldSet, Object, Span};

    use super::*;

    #[test]
    fn well_enough_specified() {
        let mut input = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
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
        }
        .into();

        run_pass(&mut input).unwrap();
    }

    #[test]
    fn not_enough_specified() {
        let mut input = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyRegister".into(),
                size_bits: 9,
                ..Default::default()
            })],
        }
        .into();

        assert_eq!(
            run_pass(&mut input).unwrap_err().to_string(),
            "No byte order is specified for fieldset `MyRegister` while it's big enough that byte order is important. Specify it on the fieldset or in the device config"
        );
    }

    #[test]
    fn not_enough_specified_but_global_config() {
        let global_config = DeviceConfig {
            byte_order: Some(ByteOrder::LE),
            ..Default::default()
        };

        let mut input = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: global_config,
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyRegister".into(),
                size_bits: 9,
                ..Default::default()
            })],
        }
        .into();

        run_pass(&mut input).unwrap();
    }
}
