use block_transform::generate_block;
use enum_transform::generate_enum;
use field_set_enum_generator::generate_field_set_enum;
use field_set_transform::generate_field_set;

use super::{Device, Enum};

mod block_transform;
mod enum_transform;
mod field_set_enum_generator;
mod field_set_transform;

pub fn transform(device: Device) -> String {
    let mut output = String::new();

    for block in &device.blocks {
        output += &generate_block(
            block,
            &device.internal_address_type,
            &device.register_address_type,
        );
    }

    let mut field_set_output = String::new();
    for field_set in &device.field_sets {
        field_set_output += &generate_field_set(field_set, device.defmt_feature.as_deref());
    }

    field_set_output +=
        &generate_field_set_enum(&device.field_sets, device.defmt_feature.as_deref());

    output += &format!(
        "
        /// Module containing the generated fieldsets of the registers and commands
        pub mod field_sets {{
            use super::*;

            {field_set_output}
        }}
    "
    );

    for enum_value in &device.enums {
        output += &generate_enum(enum_value, device.defmt_feature.as_deref());
    }

    output
}
