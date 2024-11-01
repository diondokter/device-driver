#[test]
fn same_output() {
    let dsl_text = include_str!("test-device.dsl");
    let dsl_output =
        device_driver_generation::_private_transform_dsl_mir(syn::parse_str(dsl_text).unwrap())
            .unwrap();

    let json_text = include_str!("test-device.json");
    let json_output = device_driver_generation::_private_transform_json_mir(json_text).unwrap();

    let yaml_text = include_str!("test-device.yaml");
    let yaml_output = device_driver_generation::_private_transform_yaml_mir(yaml_text).unwrap();

    let toml_text = include_str!("test-device.toml");
    let toml_output = device_driver_generation::_private_transform_toml_mir(toml_text).unwrap();

    pretty_assertions::assert_eq!(dsl_output, json_output);
    pretty_assertions::assert_eq!(dsl_output, yaml_output);
    pretty_assertions::assert_eq!(dsl_output, toml_output);
}
