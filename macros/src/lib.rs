#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

use proc_macro::TokenStream;

mod from_file;
mod from_macro;

#[proc_macro_attribute]
pub fn implement_device_from_file(attr: TokenStream, item: TokenStream) -> TokenStream {
    from_file::implement_device_from_file(attr, item)
}

#[proc_macro]
pub fn implement_device(item: TokenStream) -> TokenStream {
    from_macro::implement_device(item.into()).into()
}
