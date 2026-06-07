use std::collections::HashSet;

use crate::{
    model::{Manifest, Unique, UniqueId},
    passes::{Assumption, Pass},
};
use device_driver_diagnostics::{Diagnostics, DynError, errors::ZeroStrideRepeat};

pub struct RepeatZeroStrideRejected;

impl Pass for RepeatZeroStrideRejected {
    const ASSUMPTIONS_MADE: &[Assumption] = &[];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[];

    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<UniqueId>, DynError> {
        let mut removals = HashSet::new();

        for object in manifest.iter_objects() {
            let Some(repeat) = object.repeat() else {
                continue;
            };

            if repeat.stride == 0 {
                diagnostics.add(ZeroStrideRepeat {
                    stride: repeat.stride.span,
                });
                removals.insert(object.id());
            }
        }

        Ok(removals)
    }
}
