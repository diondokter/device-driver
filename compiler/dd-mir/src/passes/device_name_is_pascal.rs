use std::collections::HashSet;

use convert_case::Casing;

use crate::{
    model::{Manifest, ObjectId},
    passes::{Assumption, Pass},
};
use device_driver_diagnostics::{
    Diagnostics, DynError,
    errors::{DeviceNameNotPascal, InvalidIdentifier},
};

pub struct DeviceNameIsPascal;

impl Pass for DeviceNameIsPascal {
    const ASSUMPTIONS_MADE: &[Assumption] = &[];
    const ASSUMPTIONS_RELEASED: &[Assumption] = &[];

    fn run_pass(
        manifest: &mut Manifest,
        diagnostics: &mut Diagnostics,
    ) -> Result<HashSet<ObjectId>, DynError> {
        let mut removals = HashSet::new();

        for (device_id, device) in manifest.devices.iter_enumerated_mut() {
            let lenient_pascal_boundaries =
                convert_case::Boundary::defaults_from("aA:AAa:_:-: :a1:A1:1A");
            let lenient_pascal_case = convert_case::Case::Custom {
                boundaries: &lenient_pascal_boundaries,
                pattern: convert_case::Pattern::Capital,
                delimiter: "",
            };

            if let Err(e) = device
                .name
                .apply_boundaries(&lenient_pascal_boundaries)
                .check_validity()
            {
                diagnostics.add(InvalidIdentifier::new(e, device.name.span));
                removals.insert(device_id.into());
                continue;
            }

            let converted_driver_name = &device.name.original().to_case(lenient_pascal_case);

            if device.name.value.original().as_str() != converted_driver_name {
                diagnostics.add(DeviceNameNotPascal {
                    device_name: device.name.span,
                    suggestion: converted_driver_name.clone(),
                });
            }
        }

        Ok(removals)
    }
}
