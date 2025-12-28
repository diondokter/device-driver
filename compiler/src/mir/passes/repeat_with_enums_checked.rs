use std::collections::HashSet;

use miette::SourceSpan;

use crate::{
    mir::{
        Enum, LendingIterator, Manifest, Object, Repeat, RepeatSource, Unique,
        passes::search_object,
    },
    reporting::{
        Diagnostics,
        errors::{ReferencedObjectDoesNotExist, RepeatEnumWithCatchAll},
    },
};

/// Checks if the enums referenced by repeats actually exist
pub fn run_pass(manifest: &mut Manifest, diagnostics: &mut Diagnostics) {
    let mut bad_object_repeat = HashSet::new();
    let mut bad_field_repeat = HashSet::new();

    for object in manifest.iter_objects() {
        if let Some(repeat) = object.repeat().as_ref()
            && !repeat_is_ok(repeat, manifest, diagnostics)
        {
            bad_object_repeat.insert(object.id());
        }

        if let Object::FieldSet(fs) = object {
            for field in &fs.fields {
                if let Some(repeat) = field.repeat.as_ref()
                    && !repeat_is_ok(repeat, manifest, diagnostics)
                {
                    bad_field_repeat.insert((object.id(), field.id_with(fs.id())));
                }
            }
        }
    }

    // Second pass: Go though all repeats that have a bad enum and replace it with a count of 1.
    // This way we can still pass them on for further
    let mut iter = manifest.iter_objects_with_config_mut();
    while let Some((object, _)) = iter.next() {
        let id = object.id();
        if let Some(repeat) = object.repeat_mut()
            && bad_object_repeat.contains(&id)
        {
            repeat.source = RepeatSource::Count(1);
        }

        if let Object::FieldSet(fs) = object {
            let fs_id = fs.id();
            for field in &mut fs.fields {
                let field_id = field.id_with(fs_id.clone());
                if let Some(repeat) = field.repeat.as_mut()
                    && bad_field_repeat.contains(&(id.clone(), field_id))
                {
                    repeat.source = RepeatSource::Count(1);
                }
            }
        }
    }
}

fn repeat_is_ok(repeat: &Repeat, manifest: &Manifest, diagnostics: &mut Diagnostics) -> bool {
    let RepeatSource::Enum(repeat_enum) = &repeat.source else {
        return true;
    };

    if let Some(Object::Enum(enum_value)) = search_object(manifest, &repeat_enum) {
        if let Some(catch_all) = enum_catch_all(enum_value) {
            diagnostics.add(RepeatEnumWithCatchAll {
                repeat_enum: repeat_enum.span,
                enum_name: enum_value.name.span,
                catch_all,
            });
            false
        } else {
            true
        }
    } else {
        diagnostics.add(ReferencedObjectDoesNotExist {
            object_reference: repeat_enum.span,
        });
        false
    }
}

fn enum_catch_all(enum_value: &Enum) -> Option<SourceSpan> {
    enum_value
        .variants
        .iter()
        .find(|v| v.value.is_catch_all())
        .map(|v| v.name.span)
}
