use block_transform::generate_block;
use enum_transform::generate_enum;
use field_set_enum_generator::generate_field_set_enum;
use field_set_transform::generate_field_set;
use proc_macro2::TokenStream;
use quote::quote;

use super::{Device, Enum};

mod block_transform;
mod enum_transform;
mod field_set_enum_generator;
mod field_set_transform;

pub fn transform(device: Device) -> TokenStream {
    let mut tokens = TokenStream::new();

    for block in &device.blocks {
        tokens.extend(generate_block(
            block,
            &device.internal_address_type,
            &device.register_address_type,
        ));
    }

    let mut field_set_tokens = TokenStream::new();
    for field_set in &device.field_sets {
        field_set_tokens.extend(generate_field_set(
            field_set,
            device.defmt_feature.as_deref(),
        ));
    }

    field_set_tokens.extend(generate_field_set_enum(
        &device.field_sets,
        device.defmt_feature.as_deref(),
    ));

    tokens.extend(quote! {
        pub mod field_sets {
            use super::*;

            #field_set_tokens
        }
    });

    for enum_value in &device.enums {
        tokens.extend(generate_enum(enum_value, device.defmt_feature.as_deref()));
    }

    tokens
}
