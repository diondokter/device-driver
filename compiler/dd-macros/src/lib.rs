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
use syn::{LitStr, Token, bracketed, punctuated::Punctuated};

/// Macro to implement the device driver.
///
/// ## Usage:
///
/// Manifest:
/// ```rust,ignore
/// # use device_driver_macros::create_device;
/// compile!(
///     options: [],
///     manifest: "path/to/manifest/file.ddsl"
/// );
/// ```
///
/// Inline ddsl (not recommended, only used for testing):
/// ```rust,ignore
/// # use device_driver_macros::create_device;
/// compile!(
///     options: [],
///     ddsl: "
///         // DDSL input
///     "
/// );
/// ```
#[proc_macro]
pub fn compile(item: TokenStream) -> TokenStream {
    let input = match syn::parse::<Input>(item) {
        Ok(i) => i,
        Err(e) => return e.into_compile_error().into(),
    };

    match try_create_device(input) {
        Ok(tokens) => tokens,
        Err(e) => syn::Error::new(Span::call_site(), format!("{e:#}"))
            .into_compile_error()
            .into(),
    }
}

fn try_create_device(input: Input) -> Result<TokenStream, DynError> {
    let (source, source_path) = match input.source {
        Source::Ddsl(source_lit) => (source_lit.value(), source_lit.span().file()),
        Source::Manifest(path) => {
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

enum Source {
    Ddsl(LitStr),
    Manifest(LitStr),
}

struct Input {
    source: Source,
    compile_options: CompileOptions,
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut compile_options = None;
        let mut source: Option<Source> = None;

        let mut first_loop = true;

        loop {
            if first_loop {
                first_loop = false;
            } else {
                if !input.is_empty() {
                    input.parse::<Token![,]>()?;
                }
            }

            if input.is_empty()
                && let Some(source) = source
            {
                return Ok(Input {
                    source,
                    compile_options: compile_options
                        .unwrap_or_else(|| Target::Rust.get_compile_options()),
                });
            }

            let look = input.lookahead1();

            if compile_options.is_none() && look.peek(kw::options) {
                let compile_options = compile_options.insert(Target::Rust.get_compile_options());

                input.parse::<kw::options>()?;
                input.parse::<syn::Token![:]>()?;

                let content;
                bracketed!(content in input);
                let options = Punctuated::<LitStr, syn::Token![,]>::parse_terminated(&content)?;

                for option in options {
                    let option_value = option.value();
                    let Some((key, value)) = option_value.split_once('=') else {
                        return Err(syn::Error::new_spanned(
                            option,
                            "Compile option must be in format: <key>=<value>",
                        ));
                    };

                    if !compile_options.add(key, value.to_string()) {
                        return Err(syn::Error::new_spanned(
                            option,
                            if compile_options.possible_options().contains(&key) {
                                "Duplicate option".into()
                            } else {
                                format!(
                                    "Compile option not expected. Expected one of these keys: {}",
                                    compile_options.possible_options().join(", ")
                                )
                            },
                        ));
                    }
                }
            } else if source.is_none() && look.peek(kw::ddsl) {
                input.parse::<kw::ddsl>()?;
                input.parse::<syn::Token![:]>()?;

                source = Some(Source::Ddsl(input.parse()?));
            } else if source.is_none() && look.peek(kw::manifest) {
                input.parse::<kw::manifest>()?;
                input.parse::<syn::Token![:]>()?;

                source = Some(Source::Manifest(input.parse()?));
            } else {
                return Err(look.error());
            }
        }
    }
}

mod kw {
    syn::custom_keyword!(options);
    syn::custom_keyword!(ddsl);
    syn::custom_keyword!(manifest);
}
