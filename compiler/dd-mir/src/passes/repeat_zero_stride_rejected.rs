use std::collections::HashSet;

use crate::model::{Manifest, Unique, UniqueId};
use device_driver_diagnostics::{Diagnostics, errors::ZeroStrideRepeat};

pub fn run_pass(manifest: &mut Manifest, diagnostics: &mut Diagnostics) -> HashSet<UniqueId> {
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

    removals
}
