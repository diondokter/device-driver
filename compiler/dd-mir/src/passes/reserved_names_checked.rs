use std::collections::{HashMap, HashSet};

use convert_case::Case;
use device_driver_common::{identifier::RuntimeType, specifiers::Access};
use device_driver_diagnostics::{
    Diagnostics, DynError,
    errors::{FieldSetterNameCollision, ReservedOperationNameUsed},
};

use crate::{
    model::{FieldSet, LendingIterator, Manifest, Object, Unique, UniqueId},
    passes::Pass,
};

use super::Assumption;

pub struct ReservedNamesChecked;

impl Pass for ReservedNamesChecked {
    const ASSUMPTIONS_MADE: &[Assumption] = &[];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[];

    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<UniqueId>, DynError> {
        let mut removals = HashSet::new();

        let mut iter = manifest.iter_objects_with_config_mut();
        while let Some((object, _)) = iter.next() {
            let new_removals = match object {
                Object::Device(device) => {
                    check_block_reserved_names(device.iter_objects(), diagnostics)
                }
                Object::Block(block) => {
                    check_block_reserved_names(block.iter_objects(), diagnostics)
                }
                Object::FieldSet(field_set) => {
                    check_field_names(field_set, diagnostics);
                    HashSet::new()
                }
                _ => HashSet::new(),
            };
            removals.extend(new_removals);
        }

        Ok(removals)
    }
}

fn check_block_reserved_names<'a>(
    objects: impl Iterator<Item = &'a Object>,
    diagnostics: &mut Diagnostics,
) -> HashSet<UniqueId> {
    let mut removals = HashSet::new();

    const RESERVED_NAMES: &[&str] = &["new", "init", "deinit", "destroy"];

    for object in objects {
        let object_operation_name = object.name().to_case(convert_case::Case::Snake);

        if object
            .name()
            .id_type()
            .shares_namespace_with(RuntimeType::Operation)
            && RESERVED_NAMES.contains(&object_operation_name.as_str())
        {
            removals.insert(object.id());
            diagnostics.add(ReservedOperationNameUsed {
                name: object.name_span(),
                operation_name: object_operation_name,
                reserved_names: RESERVED_NAMES,
            });
        }
    }

    removals
}

fn check_field_names(field_set: &mut FieldSet, diagnostics: &mut Diagnostics) {
    let field_names: HashMap<_, _> = field_set
        .fields
        .iter()
        .map(|field| (field.name.to_case(Case::Snake), field.name.span))
        .collect();

    for field in &mut field_set.fields {
        if !field.access.is_writable() {
            continue;
        }

        let setter_name = format!("set_{}", field.name.to_case(Case::Snake));

        let Some(collision_field) = field_names.get(&setter_name).copied() else {
            continue;
        };

        // The setter collides with another field. To fix it for further compilation, we set it to read only so the setter is not generated
        field.access = Access::RO;

        diagnostics.add(FieldSetterNameCollision {
            field: field.name.span,
            setter_name: field.name.words_display_prepended("set".into()),
            collision_field,
        });
    }
}
