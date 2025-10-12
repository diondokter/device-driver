use std::collections::HashSet;

use crate::mir::{Device, Enum, Object, Unique};

use super::recurse_objects_mut;

/// Checks if all names are unique to prevent later name collisions.
/// If there is a collision an error is returned.
pub fn run_pass(device: &mut Device) -> miette::Result<()> {
    let mut seen_object_ids = HashSet::new();

    // Nothing must clash with the device name
    seen_object_ids.insert(device.id());

    recurse_objects_mut(&mut device.objects, &mut |object| {
        miette::ensure!(
            seen_object_ids.insert(object.id()),
            "Duplicate object name found: `{}`",
            object.name()
        );

        if let Object::FieldSet(field_set) = object {
            let mut seen_field_names = HashSet::new();
            for field in &field_set.fields {
                miette::ensure!(
                    seen_field_names.insert(field.name.clone()),
                    "Duplicate field name found in fieldset `{}`: `{}`",
                    field_set.name,
                    field.name
                );
            }
        }

        if let Object::Enum(Enum { name, variants, .. }) = object {
            let mut seen_variant_names = HashSet::new();

            for v in variants.iter() {
                miette::ensure!(
                    seen_variant_names.insert(v.id()),
                    "Duplicate field `{}` found in generated enum `{}`",
                    v.name,
                    name,
                );
            }
        }

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use convert_case::Boundary;

    use crate::mir::{Buffer, DeviceConfig, EnumVariant, Field, FieldSet, Object};

    use super::*;

    #[test]
    #[should_panic(expected = "Duplicate object name found: `MyBuffer`")]
    fn object_names_not_unique() {
        let global_config = DeviceConfig {
            name_word_boundaries: Boundary::defaults_from("-"),
            ..Default::default()
        };

        let mut start_mir = Device {
            name: "Device".into(),
            device_config: global_config,
            objects: vec![
                Object::Buffer(Buffer {
                    name: "MyBuffer".into(),
                    ..Default::default()
                }),
                Object::Buffer(Buffer {
                    name: "MyBuffer".into(),
                    ..Default::default()
                }),
            ],
        };

        run_pass(&mut start_mir).unwrap();
    }

    #[test]
    #[should_panic(expected = "Duplicate field name found in fieldset `Reg`: `field`")]
    fn field_names_not_unique() {
        let global_config = DeviceConfig {
            name_word_boundaries: Boundary::defaults_from("-"),
            ..Default::default()
        };

        let mut start_mir = Device {
            name: "Device".into(),
            device_config: global_config,
            objects: vec![Object::FieldSet(FieldSet {
                name: "Reg".into(),
                fields: vec![
                    Field {
                        name: "field".into(),
                        ..Default::default()
                    },
                    Field {
                        name: "field".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();
    }

    #[test]
    #[should_panic(expected = "Duplicate field `Variant` found in generated enum `Enum`")]
    fn duplicate_generated_enum_variants() {
        let global_config = DeviceConfig {
            name_word_boundaries: Boundary::defaults_from("-"),
            ..Default::default()
        };

        let mut start_mir = Device {
            name: "Device".into(),
            device_config: global_config,
            objects: vec![Object::Enum(Enum {
                name: "Enum".into(),
                variants: vec![
                    EnumVariant {
                        name: "Variant".into(),
                        ..Default::default()
                    },
                    EnumVariant {
                        name: "Variant".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            })],
        };

        run_pass(&mut start_mir).unwrap();
    }
}
