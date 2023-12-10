use device_driver_generation::{IntegerType, RWCapability, TypePath};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{braced, punctuated::Punctuated, Generics};

struct DeviceImpl {
    impl_generics: syn::Generics,
    device_type: syn::Type,

    registers: Punctuated<Register, syn::Token![,]>,
}

impl syn::parse::Parse for DeviceImpl {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<syn::Token![impl]>()?;

        let mut impl_generics: Generics = input.parse()?;

        let device_ident = input.parse()?;

        input.parse::<syn::AngleBracketedGenericArguments>().ok();

        impl_generics.where_clause = input.parse().ok();

        let registers;
        braced!(registers in input);

        let s = Self {
            impl_generics,
            device_type: device_ident,
            registers: registers.parse_terminated(Register::parse, syn::Token![,])?,
        };

        if let Some(address_type) = s.registers.first().map(|r| &r.address_type) {
            for other_address_type in s.registers.iter().map(|r| &r.address_type) {
                if *other_address_type != *address_type {
                    return Err(syn::Error::new(
                        other_address_type.span(),
                        format!("All registers must have the same address type. Previous type was `{}` and this is `{}`", address_type, other_address_type),
                    ));
                }
            }
        }

        Ok(s)
    }
}

struct Register {
    name: syn::Ident,
    rw_capability: RWCapability,
    address_type: syn::Ident,
    address_value: u64,
    size_bits_value: u64,
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
                let rw_capability_value_ident = contents.parse::<syn::Ident>()?;
                let value = rw_capability_value_ident
                    .to_string()
                    .as_str()
                    .try_into()
                    .map_err(|e| {
                        syn::Error::new(rw_capability_value_ident.span(), format!("{e}"))
                    })?;
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
                let value = contents.parse::<syn::LitInt>()?.base10_parse()?;
                contents.parse::<syn::Token![;]>()?;
                value
            },
            size_bits_value: {
                contents.parse::<syn::Token![const]>()?;
                contents.parse::<kw::SIZE_BITS>()?;
                contents.parse::<syn::Token![:]>()?;
                contents.parse::<syn::Type>()?;
                contents.parse::<syn::Token![=]>()?;
                let value = contents.parse::<syn::LitInt>()?.base10_parse()?;
                contents.parse::<syn::Token![;]>()?;
                value
            },
            fields: contents.parse_terminated(Field::parse, syn::Token![,])?,
        })
    }
}

struct Field {
    name: syn::Ident,
    register_type: IntegerType,
    conversion_type: ConversionType,
    bit_start: u32,
    bit_end: u32,
}

impl syn::parse::Parse for Field {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            register_type: {
                input.parse::<syn::Token![:]>()?;
                let register_type_ident = input.parse::<syn::Ident>()?;
                register_type_ident
                    .to_string()
                    .as_str()
                    .try_into()
                    .map_err(|e| syn::Error::new(register_type_ident.span(), format!("{e}")))?
            },
            conversion_type: input.parse()?,
            bit_start: {
                input.parse::<syn::Token![=]>()?;
                input.parse::<syn::LitInt>()?.base10_parse()?
            },
            bit_end: {
                input.parse::<syn::Token![..]>()?;
                input.parse::<syn::LitInt>()?.base10_parse()?
            },
        })
    }
}

enum ConversionType {
    None,
    Existing(syn::Path),
    Enum(syn::ItemEnum),
}

impl syn::parse::Parse for ConversionType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.parse::<syn::Token![as]>().is_ok() {
            if input.peek(syn::Token![enum]) {
                Ok(Self::Enum(input.parse()?))
            } else {
                Ok(Self::Existing(input.parse()?))
            }
        } else {
            Ok(Self::None)
        }
    }
}

mod kw {
    syn::custom_keyword!(register);
    syn::custom_keyword!(RWCapability);
    syn::custom_keyword!(ADDRESS);
    syn::custom_keyword!(SIZE_BITS);
}

pub fn implement_registers(item: TokenStream) -> TokenStream {
    let device_impl = match syn::parse2::<DeviceImpl>(item) {
        Ok(device_impl) => device_impl,
        Err(e) => return e.into_compile_error(),
    };

    let address_type = match device_impl
        .registers
        .first()
        .map(|r| r.address_type.to_string().as_str().try_into())
        .transpose()
    {
        Ok(Some(address_type)) => address_type,
        Ok(None) => IntegerType::default(),
        Err(e) => {
            return syn::Error::new(device_impl.registers[0].address_type.span(), format!("{e}"))
                .into_compile_error();
        }
    };

    let device = device_driver_generation::Device {
        address_type,
        registers: device_impl
            .registers
            .iter()
            .map(|r| device_driver_generation::Register {
                name: r.name.to_string(),
                rw_capability: r.rw_capability,
                address: r.address_value,
                size_bits: r.size_bits_value,
                fields: r
                    .fields
                    .iter()
                    .map(|f| device_driver_generation::Field {
                        name: f.name.to_string(),
                        register_type: f.register_type,
                        conversion_type: match &f.conversion_type {
                            ConversionType::None => None,
                            ConversionType::Existing(path) => {
                                Some(device_driver_generation::TypePathOrEnum::TypePath(
                                    TypePath(path.to_token_stream().to_string()),
                                ))
                            }
                            ConversionType::Enum(enum_def) => {
                                Some(device_driver_generation::TypePathOrEnum::Enum(
                                    FromIterator::from_iter(enum_def.variants.iter().map(|var| {
                                        (
                                            var.ident.to_string(),
                                            var.discriminant.as_ref().map(|d| match &d.1 {
                                                syn::Expr::Lit(syn::ExprLit {
                                                    lit: syn::Lit::Int(lit_int),
                                                    ..
                                                }) => lit_int.base10_parse().unwrap(),
                                                _ => unreachable!(),
                                            }),
                                        )
                                    })),
                                ))
                            }
                        },
                        start: f.bit_start,
                        end: f.bit_end,
                    })
                    .collect::<Vec<_>>()
                    .into(),
            })
            .collect::<Vec<_>>()
            .into(),
    };

    let item = syn::ItemImpl {
        attrs: Default::default(),
        defaultness: Default::default(),
        unsafety: Default::default(),
        impl_token: Default::default(),
        generics: device_impl.impl_generics,
        trait_: Default::default(),
        self_ty: Box::new(device_impl.device_type),
        brace_token: Default::default(),
        items: Default::default(),
    };

    proc_macro2::TokenStream::from_iter([
        device.generate_device_impl(item),
        device.generate_definitions(),
    ])
}
