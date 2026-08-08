use std::{collections::HashSet, num::NonZero};

use device_driver_common::{
    span::Span,
    specifiers::{Repeat, RepeatSource},
};

use crate::{
    model::{Enum, LendingIterator, Manifest, Object, Unique, UniqueId},
    passes::{Assumption, Pass},
    search_object,
};
use device_driver_diagnostics::{
    Diagnostics, DynError,
    errors::{ReferencedObjectDoesNotExist, RepeatEnumWithCatchAll, RepeatMathOverflow},
};

/// Checks if the enums referenced by repeats actually exist and that the enum is suitable to be used as a repeat source
pub struct RepeatMathChecked;

impl Pass for RepeatMathChecked {
    const ASSUMPTIONS_MADE: &[Assumption] = &[Assumption::NamesUnique, Assumption::EnumsNotEmpty];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[
        Assumption::RepeatEnumRefValid,
        Assumption::RepeatMathChecked,
    ];

    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<UniqueId>, DynError> {
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
                repeat.source.value = RepeatSource::Count(NonZero::new(1).unwrap());
                repeat.stride.value = 1;
            }

            if let Object::FieldSet(fs) = object {
                let fs_id = fs.id();
                for field in &mut fs.fields {
                    let field_id = field.id_with(fs_id.clone());
                    if let Some(repeat) = field.repeat.as_mut()
                        && bad_field_repeat.contains(&(id.clone(), field_id))
                    {
                        repeat.source.value = RepeatSource::Count(NonZero::new(1).unwrap());
                        repeat.stride.value = 1;
                    }
                }
            }
        }

        Ok(Default::default())
    }
}

fn repeat_is_ok(repeat: &Repeat, manifest: &Manifest, diagnostics: &mut Diagnostics) -> bool {
    let (biggest_raw_value, biggest_value_span) = match &repeat.source.value {
        RepeatSource::Enum(repeat_enum) => {
            let Some(Object::Enum(enum_value)) = search_object(manifest, repeat_enum) else {
                diagnostics.add(ReferencedObjectDoesNotExist {
                    object_reference: repeat.source.span,
                });
                return false;
            };

            if let Some(catch_all) = enum_catch_all(enum_value) {
                diagnostics.add(RepeatEnumWithCatchAll {
                    repeat_enum: repeat.source.span,
                    enum_name: enum_value.name.span,
                    catch_all,
                });
                return false;
            }

            enum_value
                .iter_variants_with_discriminant()
                .map(|(discr, v)| (discr, v.span))
                .max_by_key(|(discr, _)| (*discr * repeat.stride.value).abs())
                .expect("enums are not empty")
        }
        RepeatSource::Count(count) => ((count.get() - 1) as i128, repeat.source.span),
    };

    if i32::try_from(biggest_raw_value * repeat.stride.value).is_err() {
        diagnostics.add(RepeatMathOverflow {
            repeat_span: repeat.span,
            max_value_span: biggest_value_span,
            max_value: biggest_raw_value,
            stride: repeat.stride.value,
        });

        return false;
    }

    true
}

fn enum_catch_all(enum_value: &Enum) -> Option<Span> {
    enum_value
        .variants
        .iter()
        .find(|v| v.value.is_catch_all())
        .map(|v| v.name.span)
}
