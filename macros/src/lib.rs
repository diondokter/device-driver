#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

use std::{
    fs::File,
    io::{Read, stdout},
    path::{Path, PathBuf},
};

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{LitStr, braced, spanned::Spanned};

/// Macro to implement the device driver.
///
/// ## Usage:
///
/// Inline:
/// ```rust,ignore
/// # use device_driver_macros::create_device;
/// create_device!(
///     kdl: {
///         // KDL input
///     }
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
    let input = match syn::parse::<Input>(item) {
        Ok(i) => i,
        Err(e) => return e.into_compile_error().into(),
    };

    match input.generation_type {
        GenerationType::Kdl(tokens) => {
            let (file_contents, span) = if rustversion::cfg!(nightly) {
                std::fs::read_to_string(Path::new(&tokens.span().file()))
                    .map(|fc| (fc.replace("\r\n", "\n"), Some(tokens.span().byte_range())))
                    .unwrap_or_else(|_| (tokens.to_string(), None))
            } else {
                (tokens.to_string(), None)
            };

            let (output, diagnostics) = device_driver_generation::transform_kdl(
                &file_contents,
                span.map(device_driver_generation::miette::SourceSpan::from),
                &Path::new(&tokens.span().file()),
            );

            diagnostics.print_to(stdout().lock()).unwrap();

            output.parse().unwrap()
        }
        GenerationType::Manifest(path) => {
            let result: Result<String, syn::Error> = (|| {
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
                            format!(
                                "Could not open the manifest file at '{}': {e}",
                                path.display()
                            ),
                        )
                    })?
                    .read_to_string(&mut file_contents)
                    .unwrap();

                let (output, diagnostics) =
                    device_driver_generation::transform_kdl(&file_contents, None, &path);

                diagnostics.print_to(stdout().lock()).unwrap();

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
    Kdl(proc_macro2::TokenStream),
    Manifest(LitStr),
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let look = input.lookahead1();

        if look.peek(kw::kdl) {
            input.parse::<kw::kdl>()?;
            input.parse::<syn::Token![:]>()?;

            let braced;
            braced!(braced in input);

            let tokens = braced.parse()?;

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
