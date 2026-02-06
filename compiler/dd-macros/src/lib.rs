#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

use std::{
    fs::File,
    io::{Read, stderr},
    path::{Path, PathBuf},
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
///     kdl: "
///         // KDL input
///     "
/// );
/// ```
///
/// Manifest:
/// ```rust,ignore
/// # use device_driver_macros::create_device;
/// create_device!(
///     manifest: "path/to/manifest/file.kdl"
/// );
/// ```
#[proc_macro]
pub fn create_device(item: TokenStream) -> TokenStream {
    device_driver_diagnostics::set_miette_hook(true);

    let input = match syn::parse::<Input>(item) {
        Ok(i) => i,
        Err(e) => return e.into_compile_error().into(),
    };

    match input.generation_type {
        GenerationType::Kdl(kdl_input) => {
            let (source, span) = if cfg!(feature = "nightly") && rustversion::cfg!(nightly) {
                std::fs::read_to_string(Path::new(&kdl_input.span().file())).map_or(
                    (kdl_input.value(), None),
                    |fc| {
                        (
                            // Bug in Rust? The byte range is only correct when we remove the \r from newlines
                            fc.replace("\r\n", "\n"),
                            Some(kdl_input.span().byte_range()),
                        )
                    },
                )
            } else {
                (kdl_input.value(), None)
            };

            let (output, diagnostics) =
                device_driver_core::compile(&source, span.map(miette::SourceSpan::from));

            diagnostics
                .print_to(
                    stderr().lock(),
                    Metadata {
                        source: &source,
                        source_path: &kdl_input.span().file(),
                        term_width: None,
                        use_color: true,
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

                let (output, diagnostics) = device_driver_core::compile(&source, None);

                diagnostics
                    .print_to(
                        stderr().lock(),
                        Metadata {
                            source: &source,
                            source_path: &source_path.display().to_string(),
                            term_width: None,
                            use_color: true,
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
    Kdl(syn::LitStr),
    Manifest(LitStr),
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let look = input.lookahead1();

        if look.peek(kw::kdl) {
            input.parse::<kw::kdl>()?;
            input.parse::<syn::Token![:]>()?;

            let tokens = input.parse()?;

            Ok(Self {
                generation_type: GenerationType::Kdl(tokens),
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
    syn::custom_keyword!(kdl);
    syn::custom_keyword!(manifest);
}
