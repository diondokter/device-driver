#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

use std::{
    fs::File,
    io::{Read, stderr},
    path::PathBuf,
};

use device_driver_core::{CompileOptions, Target};
use device_driver_diagnostics::{DynError, Metadata, ResultExt};
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
    match try_create_device(item) {
        Ok(tokens) => tokens,
        Err(e) => syn::Error::new(Span::call_site(), e.to_string())
            .into_compile_error()
            .into(),
    }
}

fn try_create_device(item: TokenStream) -> Result<TokenStream, DynError> {
    let input = match syn::parse::<Input>(item) {
        Ok(i) => i,
        Err(e) => return Ok(e.into_compile_error().into()),
    };

    let (source, source_path) = match input.generation_type {
        GenerationType::Ddsl(source_lit) => (source_lit.value(), source_lit.span().file()),
        GenerationType::Manifest(path) => {
            let mut source_path = PathBuf::from(path.value());
            if source_path.is_relative() {
                let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
                source_path = manifest_dir.join(source_path);
            }

            let mut source = String::new();
            let mut file = File::open(&source_path).map_err(|e| {
                DynError::new(format!(
                    "could not open the manifest file at '{}': {e}",
                    source_path.display()
                ))
            })?;
            file.read_to_string(&mut source)
                .with_message(|| "could not read manifest file")?;
            (source, source_path.display().to_string())
        }
    };

    let (output, diagnostics) =
        device_driver_core::compile(&source, Target::Rust, input.compile_options)?;

    diagnostics
        .print_to(
            stderr().lock(),
            Metadata {
                source: &source,
                source_path: &source_path,
                term_width: None,
                ansi: true,
                unicode: true,
                anonymized_line_numbers: false,
            },
        )
        .unwrap();
    output
        .parse()
        .map_err(|e: proc_macro::LexError| DynError::new(e.to_string()))
        .with_message(|| "could not parse the output")
}

struct Input {
    generation_type: GenerationType,
    compile_options: CompileOptions,
}

enum GenerationType {
    Ddsl(syn::LitStr),
    Manifest(LitStr),
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut compile_options = Target::Rust.get_compile_options();
        assert!(compile_options.add("defmt-feature", "defmt".into())); // TODO: Parse options

        let look = input.lookahead1();

        if look.peek(kw::ddsl) {
            input.parse::<kw::ddsl>()?;
            input.parse::<syn::Token![:]>()?;

            let tokens = input.parse()?;

            Ok(Self {
                generation_type: GenerationType::Ddsl(tokens),
                compile_options,
            })
        } else if look.peek(kw::manifest) {
            input.parse::<kw::manifest>()?;
            input.parse::<syn::Token![:]>()?;

            let path = input.parse()?;

            Ok(Self {
                generation_type: GenerationType::Manifest(path),
                compile_options,
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
