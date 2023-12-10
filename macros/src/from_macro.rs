use proc_macro2::TokenStream;
use syn::{braced, punctuated::Punctuated, Generics};

#[derive(Debug)]
struct DeviceImpl {
    impl_generics: syn::Generics,
    device_ident: syn::Ident,
    type_generics: syn::AngleBracketedGenericArguments,

    registers: Punctuated<Register, syn::Token![,]>,
}

impl syn::parse::Parse for DeviceImpl {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<syn::Token![impl]>()?;

        let mut impl_generics = Generics {
            lt_token: input.parse()?,
            params: {
                let p = Punctuated::parse_separated_nonempty(input)?;
                input.parse::<syn::Token![,]>().ok();
                p
            },
            gt_token: input.parse()?,
            where_clause: None,
        };
        let device_ident = input.parse()?;

        let type_generics = input.parse()?;

        impl_generics.where_clause = input.parse()?;

        let registers;
        braced!(registers in input);

        Ok(Self {
            impl_generics,
            device_ident,
            type_generics,
            registers: registers.parse_terminated(Register::parse, syn::Token![,])?,
        })
    }
}

#[derive(Debug)]
struct Register {
    name: syn::Ident,
    rw_capability: syn::Ident,
    address_type: syn::Ident,
    address_value: syn::LitInt,
    size_bytes_value: syn::LitInt,
    fields: Punctuated<Field, syn::Token![,]>,
}

impl syn::parse::Parse for Register {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<kw::register>()?;

        let name = input.parse()?;

        let contents;
        braced!(contents in input);

        Ok(Self {
            name,
            rw_capability: {
                contents.parse::<syn::Token![type]>()?;
                contents.parse::<kw::RWCapability>()?;
                contents.parse::<syn::Token![=]>()?;
                let value = contents.parse()?;
                contents.parse::<syn::Token![;]>()?;
                value
            },
            address_type: {
                contents.parse::<syn::Token![const]>()?;
                contents.parse::<kw::ADDRESS>()?;
                contents.parse::<syn::Token![:]>()?;
                contents.parse()?
            },
            address_value: {
                contents.parse::<syn::Token![=]>()?;
                let value = contents.parse()?;
                contents.parse::<syn::Token![;]>()?;
                value
            },
            size_bytes_value: {
                contents.parse::<syn::Token![const]>()?;
                contents.parse::<kw::SIZE_BYTES>()?;
                contents.parse::<syn::Token![:]>()?;
                contents.parse::<syn::Type>()?;
                contents.parse::<syn::Token![=]>()?;
                let value = contents.parse()?;
                contents.parse::<syn::Token![;]>()?;
                value
            },
            fields: contents.parse_terminated(Field::parse, syn::Token![,])?,
        })
    }
}

#[derive(Debug)]
struct Field {
    name: syn::Ident,
    integer: syn::Ident,
    conversion_type: Option<syn::Ident>,
    bit_start: syn::LitInt,
    bit_end: syn::LitInt,
}

impl syn::parse::Parse for Field {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            integer: {
                input.parse::<syn::Token![:]>()?;
                input.parse()?
            },
            conversion_type: input
                .parse::<syn::Token![as]>()
                .ok()
                .map(|_| input.parse())
                .transpose()?,
            bit_start: {
                input.parse::<syn::Token![=]>()?;
                input.parse()?
            },
            bit_end: {
                input.parse::<syn::Token![..]>()?;
                input.parse()?
            },
        })
    }
}

mod kw {
    syn::custom_keyword!(register);
    syn::custom_keyword!(RWCapability);
    syn::custom_keyword!(ADDRESS);
    syn::custom_keyword!(SIZE_BYTES);
}

pub fn implement_registers(item: TokenStream) -> TokenStream {
    let impl_ = match syn::parse2::<DeviceImpl>(item) {
        Ok(device_impl) => device_impl,
        Err(e) => return e.into_compile_error(),
    };

    todo!("{impl_:?}");
}
