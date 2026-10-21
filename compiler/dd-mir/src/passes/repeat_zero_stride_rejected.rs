use std::collections::HashSet;

use crate::{
    model::{Manifest, ObjectId},
    passes::{Assumption, Pass},
};
use device_driver_diagnostics::{Diagnostics, DynError, errors::ZeroStrideRepeat};

pub struct RepeatZeroStrideRejected;

impl Pass for RepeatZeroStrideRejected {
    const ASSUMPTIONS_MADE: &[Assumption] = &[];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[Assumption::RepeatStrideNonZero];

    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<ObjectId>, DynError> {
        let mut removals = HashSet::new();

        for (object_id, object) in manifest.objects_enumerated() {
            let Some(repeat) = object.repeat() else {
                continue;
            };

            if repeat.stride == 0 {
                diagnostics.add(ZeroStrideRepeat {
                    stride: repeat.stride.span,
                });
                removals.insert(object_id);
            }
        }

        Ok(removals)
    }
}
