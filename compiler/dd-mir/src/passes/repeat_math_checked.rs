use std::{collections::HashSet, num::NonZero};

use device_driver_common::{
    span::Span,
    specifiers::{Repeat, RepeatSource},
};

use crate::{
    model::{Enum, Manifest, Object, ObjectId},
    passes::{Assumption, Pass},
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
    ) -> Result<HashSet<ObjectId>, DynError> {
        let all_objects = manifest.object_ids().collect::<Vec<_>>();

        for object_id in all_objects {
            let object = manifest.object(object_id).unwrap();

            if let Some(repeat) = object.repeat().as_ref()
                && !repeat_is_ok(repeat, manifest, diagnostics)
            {
                let mut object = manifest.object_mut(object_id).unwrap();
                let repeat = object.repeat_mut().unwrap();
                repeat.source.value = RepeatSource::Count(NonZero::new(1).unwrap());
                repeat.stride.value = 1;
            }
        }

        Ok(Default::default())
    }
}

fn repeat_is_ok(repeat: &Repeat, manifest: &Manifest, diagnostics: &mut Diagnostics) -> bool {
    let (biggest_raw_value, biggest_value_span) = match &repeat.source.value {
        RepeatSource::Enum(repeat_enum) => {
            let Some(Object::Enum(enum_value)) = manifest.search_object(repeat_enum) else {
                diagnostics.add(ReferencedObjectDoesNotExist {
                    object_reference: repeat.source.span,
                });
                return false;
            };

            if let Some(catch_all) = enum_catch_all(enum_value, manifest) {
                diagnostics.add(RepeatEnumWithCatchAll {
                    repeat_enum: repeat.source.span,
                    enum_name: enum_value.name.span,
                    catch_all,
                });
                return false;
            }

            enum_value
                .iter_variants_with_discriminant(&manifest.enum_variants)
                .map(|(discr, v)| (discr, manifest.enum_variants.get(v).unwrap().span))
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

fn enum_catch_all(enum_value: &Enum, manifest: &Manifest) -> Option<Span> {
    enum_value.variants.iter().find_map(|v| {
        let variant = manifest.enum_variants.get(*v).unwrap();
        variant.value.is_catch_all().then_some(variant.name.span)
    })
}
