use enum_transform::generate_enum;
use field_set_transform::generate_field_set;
use proc_macro2::TokenStream;

use super::{Device, Enum};

mod enum_transform;
mod field_set_transform;

pub fn transform_lir(device: Device) -> TokenStream {
    let mut tokens = TokenStream::new();

    for field_set in &device.field_sets {
        tokens.extend(generate_field_set(field_set));
    }

    for enum_value in &device.enums {
        tokens.extend(generate_enum(enum_value));
    }

    tokens
}
