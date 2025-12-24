use crate::{
    mir::{LendingIterator, Manifest},
    reporting::{Diagnostics, errors::UnspecifiedByteOrder},
};

/// Checks if the byte order is set for all registers and commands that need it and fills it out for the ones that aren't specified
pub fn run_pass(manifest: &mut Manifest, diagnostics: &mut Diagnostics) {
    let mut iter = manifest.iter_objects_with_config_mut();
    while let Some((object, config)) = iter.next() {
        if let Some(fs) = object.as_field_set_mut() {
            if fs.byte_order.is_none() {
                fs.byte_order = config.byte_order;
            }

            if fs.size_bits > 8 && fs.byte_order.is_none() {
                diagnostics.add(UnspecifiedByteOrder {
                    fieldset_name: fs.name.span,
                });
            }

            // Even if not required, fill in a byte order so we can always unwrap it later
            fs.byte_order.get_or_insert(crate::mir::ByteOrder::LE);
        }
    }
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
                    name: "MyRegister".to_owned().with_dummy_span(),
                    size_bits: 8.with_dummy_span(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: "MyRegister2".to_owned().with_dummy_span(),
                    size_bits: 9.with_dummy_span(),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
        }
        .into();

        let mut d = Diagnostics::new();
        run_pass(&mut input, &mut d);
        assert!(!d.has_error());
    }

    #[test]
    fn not_enough_specified() {
        let mut input = Device {
            description: String::new(),
            name: "Device".to_owned().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: "MyRegister".to_owned().with_dummy_span(),
                size_bits: 9.with_dummy_span(),
                ..Default::default()
            })],
        }
        .into();

        let mut d = Diagnostics::new();
        run_pass(&mut input, &mut d);
        assert!(d.has_error());
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
                name: "MyRegister".to_owned().with_dummy_span(),
                size_bits: 9.with_dummy_span(),
                ..Default::default()
            })],
        }
        .into();

        let mut d = Diagnostics::new();
        run_pass(&mut input, &mut d);
        assert!(!d.has_error());
    }
}
