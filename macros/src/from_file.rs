#[cfg(any(feature = "json", feature = "yaml"))]
use std::io::BufReader;

use std::fs::File;

use device_driver_generation::Device;
use proc_macro::TokenStream;
use syn::{spanned::Spanned, Ident, LitStr};

struct AttrParams {
    key_value: Option<KeyValue>,
}

impl syn::parse::Parse for AttrParams {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            key_value: if input.is_empty() {
                None
            } else {
                Some(KeyValue::parse(input)?)
            },
        })
    }
}

struct KeyValue {
    key: Ident,
    value: LitStr,
}

impl syn::parse::Parse for KeyValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let key = input.parse()?;
        input.parse::<syn::Token![=]>()?;
        let value = input.parse()?;

        Ok(Self { key, value })
    }
}

pub fn implement_registers_from_file(attr: TokenStream, item: TokenStream) -> TokenStream {
    let key_value = syn::parse_macro_input!(attr as AttrParams).key_value;
    let item = syn::parse_macro_input!(item as syn::ItemImpl);

    #[allow(unreachable_code, unused_variables)]
    match key_value {
        None => syn::Error::new(
            item.span(),
            "No arguments given. Use e.g. `json = \"path/to/file.json\"",
        )
        .into_compile_error()
        .into(),
        Some(key_value) => {
            let file = match File::open(key_value.value.value()) {
                Ok(file) => file,
                Err(e) => {
                    return syn::Error::new(
                        key_value.value.span(),
                        format!(
                            "Error opening file: {e:?}.\nWorking directory is {:?}",
                            std::env::current_dir()
                        ),
                    )
                    .into_compile_error()
                    .into()
                }
            };

            let device: Device = match key_value.key.to_string().to_lowercase().as_str() {
                #[cfg(feature = "json")]
                "json" => match serde_json::from_reader(BufReader::new(file)) {
                    Ok(device) => device,
                    Err(e) => {
                        return syn::Error::new(
                            key_value.value.span(),
                            format!("Could not parse device: {e}"),
                        )
                        .into_compile_error()
                        .into()
                    }
                },
                #[cfg(feature = "yaml")]
                "yaml" | "yml" => match serde_yaml::from_reader(BufReader::new(file)) {
                    Ok(device) => device,
                    Err(e) => {
                        return syn::Error::new(
                            key_value.value.span(),
                            format!("Could not parse device: {e}"),
                        )
                        .into_compile_error()
                        .into()
                    }
                },
                key => {
                    return syn::Error::new(
                        key_value.key.span(),
                        format!("\"{key}\" is not recognized. This might mean a feature is not enabled yet."),
                    )
                    .into_compile_error()
                    .into();
                }
            };

            proc_macro2::TokenStream::from_iter([
                device.generate_device_impl(item),
                device.generate_definitions(),
            ])
            .into()
        }
    }
}
