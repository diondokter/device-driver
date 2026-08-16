use std::collections::HashSet;

use device_driver_common::specifiers::Access;
use device_driver_diagnostics::{Diagnostics, DynError, errors::UnspecifiedAccess};

use crate::{
    model::{Manifest, Object, UniqueId},
    passes::Pass,
};

use super::Assumption;

pub struct AccessSet;

impl Pass for AccessSet {
    const ASSUMPTIONS_MADE: &[Assumption] = &[];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[Assumption::AccessSet];

    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<UniqueId>, DynError> {
        set_access(manifest.default_access, &mut manifest.objects, diagnostics);
        Ok(Default::default())
    }
}

fn set_access(
    default_access: Option<Access>,
    objects: &mut [Object],
    diagnostics: &mut Diagnostics,
) {
    for object in objects {
        match object {
            Object::Device(device) => {
                let default_access = device.default_access.or(default_access);
                set_access(default_access, &mut device.objects, diagnostics);
            }
            Object::Block(block) => {
                let default_access = block.default_access.or(default_access);
                set_access(default_access, &mut block.objects, diagnostics);
            }
            Object::FieldSet(field_set) => {
                let default_access = field_set.default_access.or(default_access);
                for field in field_set.fields.iter_mut() {
                    field.access = field.access.or(default_access);

                    if field.access.is_none() {
                        field.access = Some(Access::RW);
                        diagnostics.add(UnspecifiedAccess {
                            object_name: field.name.span,
                            short_property: true,
                            properties_span: field.short_properties_span,
                        });
                    }
                }
            }
            Object::Register(register) => {
                register.access = register.access.or(default_access);

                if register.access.is_none() {
                    register.access = Some(Access::RW);
                    diagnostics.add(UnspecifiedAccess {
                        object_name: register.name.span,
                        short_property: false,
                        properties_span: register.properties_span,
                    });
                }
            }
            Object::Buffer(buffer) => {
                buffer.access = buffer.access.or(default_access);

                if buffer.access.is_none() {
                    buffer.access = Some(Access::RW);
                    diagnostics.add(UnspecifiedAccess {
                        object_name: buffer.name.span,
                        short_property: false,
                        properties_span: buffer.properties_span,
                    });
                }
            }

            Object::Field(_) => {
                // Intentionally left empty as fields are done inline in the fieldset case
            }
            Object::Command(_) | Object::Enum(_) | Object::Extern(_) => {
                // Intentionally left empty as they don't have children we care about and they don't carry an access specifier themselves
            }
        }
    }
}
