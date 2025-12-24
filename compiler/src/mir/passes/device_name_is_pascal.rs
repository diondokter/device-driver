use crate::{
    mir::{LendingIterator, Manifest, Object},
    reporting::{Diagnostics, errors::DeviceNameNotPascal},
};

pub fn run_pass(manifest: &mut Manifest, diagnostics: &mut Diagnostics) {
    let mut iter = manifest.iter_objects_with_config_mut();
    while let Some((object, _)) = iter.next() {
        let Object::Device(device) = object else {
            continue;
        };

        let lenient_pascal_converter = convert_case::Converter::new()
            .set_boundaries(&convert_case::Boundary::defaults_from(
                "aA:AAa:_:-: :a1:A1:1A",
            ))
            .set_pattern(convert_case::pattern::capital);
        let converted_driver_name = lenient_pascal_converter.convert(&device.name.value);

        if device.name.value != converted_driver_name {
            diagnostics.add(DeviceNameNotPascal {
                device_name: device.name.span,
                suggestion: converted_driver_name.clone(),
            });
            // Change the name already so we get consistent casing further along
            device.name.value = converted_driver_name;
        }
    }
}
