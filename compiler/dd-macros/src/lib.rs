#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

use std::{
    fs::File,
    io::{Read, stderr},
    path::PathBuf,
};

use device_driver_diagnostics::Metadata;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::LitStr;

/// Macro to implement the device driver.
///
/// ## Usage:
///
/// Inline:
/// ```rust,ignore
/// # use device_driver_macros::create_device;
/// create_device!(
///     ddsl: "
///         // DDSL input
///     "
/// );
/// ```
///
/// Manifest:
/// ```rust,ignore
/// # use device_driver_macros::create_device;
/// create_device!(
///     manifest: "path/to/manifest/file.ddsl"
/// );
/// ```
#[proc_macro]
pub fn create_device(item: TokenStream) -> TokenStream {
    let input = match syn::parse::<Input>(item) {
        Ok(i) => i,
        Err(e) => return e.into_compile_error().into(),
    };

    match input.generation_type {
        GenerationType::Ddsl(source_lit) => {
            let source = source_lit.value();

            let (output, diagnostics) = device_driver_core::compile(&source);

            diagnostics
                .print_to(
                    stderr().lock(),
                    Metadata {
                        source: &source,
                        source_path: &source_lit.span().file(),
                        term_width: None,
                        ansi: true,
                        unicode: true,
                        anonymized_line_numbers: false,
                    },
                )
                .unwrap();

            output.parse().unwrap()
        }
        GenerationType::Manifest(path) => {
            let result: Result<String, syn::Error> = (|| {
                let mut source_path = PathBuf::from(path.value());
                if source_path.is_relative() {
                    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
                    source_path = manifest_dir.join(source_path);
                }

                let mut source = String::new();
                File::open(&source_path)
                    .map_err(|e| {
                        syn::Error::new(
                            Span::call_site(),
                            format!(
                                "Could not open the manifest file at '{}': {e}",
                                source_path.display()
                            ),
                        )
                    })?
                    .read_to_string(&mut source)
                    .unwrap();

                let (output, diagnostics) = device_driver_core::compile(&source);

                diagnostics
                    .print_to(
                        stderr().lock(),
                        Metadata {
                            source: &source,
                            source_path: &source_path.display().to_string(),
                            term_width: None,
                            ansi: true,
                            unicode: true,
                            anonymized_line_numbers: false,
                        },
                    )
                    .unwrap();

                Ok(output)
            })();

            match result {
                Ok(tokens) => tokens.parse().unwrap(),
                Err(e) => e.into_compile_error().into(),
            }
        }
    }
}

struct Input {
    generation_type: GenerationType,
}

enum GenerationType {
    Ddsl(syn::LitStr),
    Manifest(LitStr),
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let look = input.lookahead1();

        if look.peek(kw::ddsl) {
            input.parse::<kw::ddsl>()?;
            input.parse::<syn::Token![:]>()?;

            let tokens = input.parse()?;

            Ok(Self {
                generation_type: GenerationType::Ddsl(tokens),
            })
        } else if look.peek(kw::manifest) {
            input.parse::<kw::manifest>()?;
            input.parse::<syn::Token![:]>()?;

            let path = input.parse()?;

            Ok(Self {
                generation_type: GenerationType::Manifest(path),
            })
        } else {
            Err(look.error())
        }
    }
}

mod kw {
    syn::custom_keyword!(ddsl);
    syn::custom_keyword!(manifest);
}
