use enum_transform::generate_enum;
use proc_macro2::TokenStream;

use super::{Device, Enum};

mod enum_transform;

pub fn transform_lir(device: Device) -> TokenStream {
    let mut tokens = TokenStream::new();

    for enum_value in &device.enums {
        tokens.extend(generate_enum(enum_value));
    }

    tokens
}
