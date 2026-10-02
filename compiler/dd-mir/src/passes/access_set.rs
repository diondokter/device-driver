use std::collections::HashSet;

use device_driver_common::specifiers::Access;
use device_driver_diagnostics::{Diagnostics, DynError, errors::UnspecifiedAccess};

use crate::{
    model::{Manifest, ObjectId, ObjectMut},
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
    ) -> Result<HashSet<ObjectId>, DynError> {
        let all_objects = manifest.object_ids().collect::<Vec<_>>();

        for object_id in all_objects {
            let default_access = manifest.object_default_access(object_id);

            let object = object_id.get_mut(manifest).unwrap();
            match object {
                ObjectMut::Register(val) => {
                    val.access = val.access.or(default_access);
                    if val.access.is_none() {
                        val.access = Some(Access::RW);
                        diagnostics.add(UnspecifiedAccess {
                            object_name: val.name.span,
                            short_property: false,
                            properties_span: val.properties_span,
                        });
                    }
                }
                ObjectMut::Buffer(val) => {
                    val.access = val.access.or(default_access);
                    if val.access.is_none() {
                        val.access = Some(Access::RW);
                        diagnostics.add(UnspecifiedAccess {
                            object_name: val.name.span,
                            short_property: false,
                            properties_span: val.properties_span,
                        });
                    }
                }
                ObjectMut::Field(val) => {
                    val.access = val.access.or(default_access);
                    if val.access.is_none() {
                        val.access = Some(Access::RW);
                        diagnostics.add(UnspecifiedAccess {
                            object_name: val.name.span,
                            short_property: true,
                            properties_span: val.properties_span,
                        });
                    }
                }
                ObjectMut::Device(_) => {}
                ObjectMut::Block(_) => {}
                ObjectMut::Command(_) => {}
                ObjectMut::FieldSet(_) => {}
                ObjectMut::Enum(_) => {}
                ObjectMut::Extern(_) => {}
                ObjectMut::EnumVariant(_) => {}
            }
        }

        Ok(Default::default())
    }
}
