use anyhow::ensure;

use crate::mir::Device;

pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    let lenient_pascal_converter = convert_case::Converter::new()
        .set_boundaries(&convert_case::Boundary::defaults_from(
            "aA:AAa:_:-: :a1:A1:1A",
        ))
        .set_pattern(convert_case::pattern::capital);
    let converted_driver_name = lenient_pascal_converter.convert(device.name.as_ref().unwrap());

    ensure!(
        device.name.as_ref().unwrap() == &converted_driver_name,
        "The device name must be given in PascalCase, e.g. \"{converted_driver_name}\"",
    );

    Ok(())
}
