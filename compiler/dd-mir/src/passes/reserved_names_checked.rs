use std::collections::HashSet;

use convert_case::Case;
use device_driver_common::identifier::RuntimeNamespace;
use device_driver_diagnostics::{
    Diagnostics, DynError,
    errors::{FieldSetterNameCollision, ReservedOperationNameUsed},
};

use crate::{
    model::{Manifest, Object, ObjectId, ObjectType},
    passes::Pass,
};

use super::Assumption;

const RESERVED_NAMES: &[&str] = &["new", "init", "deinit", "free"];

pub struct ReservedNamesChecked;

impl Pass for ReservedNamesChecked {
    const ASSUMPTIONS_MADE: &[Assumption] = &[Assumption::NamesValid, Assumption::AccessSet];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[];

    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<ObjectId>, DynError> {
        let mut removals = HashSet::new();

        for (object_id, object) in manifest.objects_enumerated() {
            if object
                .name()
                .namespace()
                .shares_namespace_with(RuntimeNamespace::Operation)
                || object_id.object_type() == ObjectType::Field
            {
                let object_operation_name = object.name().to_case(convert_case::Case::Snake);
                if RESERVED_NAMES.contains(&object_operation_name.as_str()) {
                    removals.insert(object_id);
                    diagnostics.add(ReservedOperationNameUsed {
                        name: object.name_span(),
                        operation_name: object_operation_name,
                        reserved_names: RESERVED_NAMES,
                    });
                    continue;
                }
            }

            // Specifically for writable fields we need to check if the `set_*` name doesn't collide with another
            if let Object::Field(field) = object {
                if !field
                    .access
                    .ok_or_else(|| DynError::new("access is not set"))?
                    .is_writable()
                {
                    continue;
                }

                let setter_name = format!("set_{}", field.name.to_case(Case::Snake));

                let Some(colliding_field) = manifest
                    .fields
                    .iter()
                    // Filter to the same namespace (so defined within the same fieldset)
                    .filter(|other_field| other_field.name.namespace() == field.name.namespace())
                    .find(|other_field| other_field.name.to_case(Case::Snake) == setter_name)
                else {
                    continue;
                };

                removals.insert(object_id);

                diagnostics.add(FieldSetterNameCollision {
                    field: field.name.span,
                    setter_name: field.name.words_display_prepended("set".into()),
                    collision_field: colliding_field.name.span,
                });
            }
        }

        Ok(removals)
    }
}
