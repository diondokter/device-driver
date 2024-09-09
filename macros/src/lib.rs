#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

use proc_macro::TokenStream;
use syn::{braced, Ident, LitStr};

#[proc_macro]
pub fn implement_device(item: TokenStream) -> TokenStream {
    let input = match syn::parse::<Input>(item) {
        Ok(i) => i,
        Err(e) => return e.into_compile_error().into(),
    };

    match input.generation_type {
        GenerationType::Dsl(tokens) => {
            device_driver_generation::transform_dsl(tokens, &input.device_name.to_string()).into()
        }
        GenerationType::Json(_) => todo!(),
        GenerationType::Yaml(_) => todo!(),
    }
}

struct Input {
    device_name: Ident,
    generation_type: GenerationType,
}

enum GenerationType {
    Dsl(proc_macro2::TokenStream),
    Json(LitStr),
    Yaml(LitStr),
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<kw::name>()?;
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
        } else if look.peek(kw::json) {
            input.parse::<kw::json>()?;
            input.parse::<syn::Token![:]>()?;

            let path = input.parse()?;

            Ok(Self {
                device_name,
                generation_type: GenerationType::Json(path),
            })
        } else if look.peek(kw::yaml) {
            input.parse::<kw::yaml>()?;
            input.parse::<syn::Token![:]>()?;

            let path = input.parse()?;

            Ok(Self {
                device_name,
                generation_type: GenerationType::Yaml(path),
            })
        } else {
            Err(look.error())
        }
    }
}

mod kw {
    syn::custom_keyword!(name);
    syn::custom_keyword!(dsl);
    syn::custom_keyword!(json);
    syn::custom_keyword!(yaml);
}
