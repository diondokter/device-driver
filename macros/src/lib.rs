#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

use std::{fs::File, io::Read, ops::Deref, path::PathBuf};

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{braced, Ident, LitStr};

/// Macro to implement the device driver.
/// 
/// ## Usage:
/// 
/// DSL:
/// ```rust
/// # use device_driver_macros::create_device;
/// create_device!(
///     device_name: MyTestDevice,
///     dsl: {
///         // DSL
///     }
/// );
/// ```
/// 
/// Manifest:
/// ```rust,ignore
/// # use device_driver_macros::create_device;
/// create_device!(
///     device_name: MyTestDevice,
///     manifest: "path/to/manifest/file.json"
/// );
/// ```
#[proc_macro]
pub fn create_device(item: TokenStream) -> TokenStream {
    let input = match syn::parse::<Input>(item) {
        Ok(i) => i,
        Err(e) => return e.into_compile_error().into(),
    };

    match input.generation_type {
        #[cfg(feature = "dsl")]
        GenerationType::Dsl(tokens) => {
            device_driver_generation::transform_dsl(tokens, &input.device_name.to_string()).into()
        }
        #[cfg(not(feature = "dsl"))]
        GenerationType::Dsl(tokens) => {
            syn::Error::new(Span::call_site(), format!("The dsl feature is not enabled"))
                .into_compile_error()
                .into()
        }
        GenerationType::Manifest(path) => {
            let result: Result<proc_macro2::TokenStream, syn::Error> = (|| {
                let mut path = PathBuf::from(path.value());
                if path.is_relative() {
                    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
                    path = manifest_dir.join(path);
                }

                let mut file_contents = String::new();
                File::open(&path)
                    .map_err(|e| {
                        syn::Error::new(
                            Span::call_site(),
                            format!("Could open the manifest file at '{}': {e}", path.display()),
                        )
                    })?
                    .read_to_string(&mut file_contents)
                    .unwrap();

                let extension =
                    path.extension()
                        .map(|ext| ext.to_string_lossy())
                        .ok_or(syn::Error::new(
                            Span::call_site(),
                            "Manifest file has no file extension",
                        ))?;

                match extension.deref() {
                    #[cfg(feature = "json")]
                    "json" => Ok(device_driver_generation::transform_json(
                        &file_contents,
                        &input.device_name.to_string(),
                    )),
                    #[cfg(not(feature = "json"))]
                    "json" => Err(syn::Error::new(
                        Span::call_site(),
                        format!("The json feature is not enabled"),
                    )),
                    #[cfg(feature = "yaml")]
                    "yaml" => Ok(device_driver_generation::transform_yaml(
                        &file_contents,
                        &input.device_name.to_string(),
                    )),
                    #[cfg(not(feature = "yaml"))]
                    "yaml" => Err(syn::Error::new(
                        Span::call_site(),
                        format!("The yaml feature is not enabled"),
                    )),
                    #[cfg(feature = "toml")]
                    "toml" => Ok(device_driver_generation::transform_toml(
                        &file_contents,
                        &input.device_name.to_string(),
                    )),
                    #[cfg(not(feature = "toml"))]
                    "toml" => Err(syn::Error::new(
                        Span::call_site(),
                        format!("The toml feature is not enabled"),
                    )),
                    "dsl" => Ok(device_driver_generation::transform_dsl(
                        syn::parse_str(&file_contents)?,
                        &input.device_name.to_string(),
                    )),
                    unknown => Err(syn::Error::new(
                        Span::call_site(),
                        format!("Unknown manifest file extension: '{unknown}'"),
                    )),
                }
            })();

            match result {
                Ok(tokens) => tokens.into(),
                Err(e) => e.into_compile_error().into(),
            }
        }
    }
}

struct Input {
    device_name: Ident,
    generation_type: GenerationType,
}

enum GenerationType {
    Dsl(proc_macro2::TokenStream),
    Manifest(LitStr),
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<kw::device_name>()?;
        input.parse::<syn::Token![:]>()?;
        let device_name = input.parse()?;
        input.parse::<syn::Token![,]>()?;

        let look = input.lookahead1();

        if look.peek(kw::dsl) {
            input.parse::<kw::dsl>()?;
            input.parse::<syn::Token![:]>()?;

            let braced;
            braced!(braced in input);

            let tokens = braced.parse()?;

            Ok(Self {
                device_name,
                generation_type: GenerationType::Dsl(tokens),
            })
        } else if look.peek(kw::manifest) {
            input.parse::<kw::manifest>()?;
            input.parse::<syn::Token![:]>()?;

            let path = input.parse()?;

            Ok(Self {
                device_name,
                generation_type: GenerationType::Manifest(path),
            })
        } else {
            Err(look.error())
        }
    }
}

mod kw {
    syn::custom_keyword!(device_name);
    syn::custom_keyword!(dsl);
    syn::custom_keyword!(manifest);
}
