use std::collections::HashSet;

use crate::{
    model::{LendingIterator, Manifest, UniqueId},
    passes::{Assumption, Pass},
};
use device_driver_common::specifiers::ByteOrder;
use device_driver_diagnostics::{Diagnostics, DynError, errors::UnspecifiedByteOrder};

/// Checks if the byte order is set for all registers and commands that need it and fills it out for the ones that aren't specified
pub struct ByteOrderSpecified;

impl Pass for ByteOrderSpecified {
    const ASSUMPTIONS_MADE: &[Assumption] = &[];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[];

    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<UniqueId>, DynError> {
        let mut iter = manifest.iter_objects_with_config_mut();
        while let Some((object, config)) = iter.next() {
            if let Some(fs) = object.as_field_set_mut() {
                if fs.byte_order.is_none() {
                    fs.byte_order = config.byte_order;
                }

                if fs.size_bytes > 1 && fs.byte_order.is_none() {
                    diagnostics.add(UnspecifiedByteOrder {
                        fieldset_name: fs.name.span,
                    });
                }

                // Even if not required, fill in a byte order so we can always unwrap it later
                fs.byte_order.get_or_insert(ByteOrder::LE);
            }
        }

        Ok(Default::default())
    }
}

#[cfg(test)]
mod tests {
    use device_driver_common::{
        identifier::Identifier,
        span::{Span, SpanExt},
        specifiers::ByteOrder,
    };

    use crate::model::{Device, DeviceConfig, FieldSet, Object};

    use super::*;

    #[test]
    fn well_enough_specified() {
        let mut input = Device {
            description: String::new(),
            name: Identifier::try_parse("Device").unwrap().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![
                Object::FieldSet(FieldSet {
                    name: Identifier::try_parse("MyRegister")
                        .unwrap()
                        .with_dummy_span(),
                    size_bytes: 1.with_dummy_span(),
                    ..Default::default()
                }),
                Object::FieldSet(FieldSet {
                    name: Identifier::try_parse("MyRegister2")
                        .unwrap()
                        .with_dummy_span(),
                    size_bytes: 2.with_dummy_span(),
                    byte_order: Some(ByteOrder::LE),
                    ..Default::default()
                }),
            ],
            span: Span::default(),
        }
        .into();

        let mut d = Diagnostics::new();
        ByteOrderSpecified::run_pass(&mut input, &mut d).unwrap();
        assert!(!d.has_error());
    }

    #[test]
    fn not_enough_specified() {
        let mut input = Device {
            description: String::new(),
            name: Identifier::try_parse("Device").unwrap().with_dummy_span(),
            device_config: Default::default(),
            objects: vec![Object::FieldSet(FieldSet {
                name: Identifier::try_parse("MyRegister")
                    .unwrap()
                    .with_dummy_span(),
                size_bytes: 2.with_dummy_span(),
                ..Default::default()
            })],
            span: Span::default(),
        }
        .into();

        let mut d = Diagnostics::new();
        ByteOrderSpecified::run_pass(&mut input, &mut d).unwrap();
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
            name: Identifier::try_parse("Device").unwrap().with_dummy_span(),
            device_config: global_config,
            objects: vec![Object::FieldSet(FieldSet {
                name: Identifier::try_parse("MyRegister")
                    .unwrap()
                    .with_dummy_span(),
                size_bytes: 2.with_dummy_span(),
                ..Default::default()
            })],
            span: Span::default(),
        }
        .into();

        let mut d = Diagnostics::new();
        ByteOrderSpecified::run_pass(&mut input, &mut d).unwrap();
        assert!(!d.has_error());
    }
}
