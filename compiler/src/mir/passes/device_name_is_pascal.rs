use convert_case::Casing;

use crate::{
    identifier::Identifier,
    mir::{LendingIterator, Manifest, Object},
    reporting::{Diagnostics, errors::DeviceNameNotPascal},
};

pub fn run_pass(manifest: &mut Manifest, diagnostics: &mut Diagnostics) {
    let mut iter = manifest.iter_objects_with_config_mut();
    while let Some((object, _)) = iter.next() {
        let Object::Device(device) = object else {
            continue;
        };

        let lenient_pascal_boundaries =
            convert_case::Boundary::defaults_from("aA:AAa:_:-: :a1:A1:1A");
        let lenient_pascal_case = convert_case::Case::Custom {
            boundaries: &lenient_pascal_boundaries,
            pattern: convert_case::Pattern::Capital,
            delim: "",
        };
        device.name.apply_boundaries(&lenient_pascal_boundaries);
        let converted_driver_name = &device.name.original().to_case(lenient_pascal_case);

        if device.name.value.original() != converted_driver_name {
            diagnostics.add(DeviceNameNotPascal {
                device_name: device.name.span,
                suggestion: converted_driver_name.clone(),
            });
            // Change the name already so we get consistent casing further along
            device.name.value = Identifier::try_parse(converted_driver_name).unwrap();
            device.name.apply_boundaries(&lenient_pascal_boundaries);
        }
    }
}
