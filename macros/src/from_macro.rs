use device_driver_generation::{
    deserialization::RegisterCollection, BaseType, EnumVariant, EnumVariantValue, RWType,
    ResetValue, TypePath,
};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    braced, punctuated::Punctuated, spanned::Spanned, Attribute, Expr, ExprLit, Generics, Lit,
};

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
    rw_type: RWType,
    address_type: syn::Ident,
    address_value: u64,
    size_bits_value: u64,
    reset_value: Option<ResetValue>,
    description: Option<String>,
    fields: Punctuated<Field, syn::Token![,]>,
}

impl syn::parse::Parse for Register {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let register_attributes = syn::Attribute::parse_outer(input)?;

        let description = doc_string_from_attrs(&register_attributes)?;

        input.parse::<kw::register>()?;

        let name = input.parse()?;

        let contents;
        braced!(contents in input);

        Ok(Self {
            name,
            rw_type: {
                contents.parse::<syn::Token![type]>()?;
                contents.parse::<kw::RWType>()?;
                contents.parse::<syn::Token![=]>()?;
                let rw_type_value_ident = contents.parse::<syn::Ident>()?;
                let value = rw_type_value_ident
                    .to_string()
                    .as_str()
                    .try_into()
                    .map_err(|e| syn::Error::new(rw_type_value_ident.span(), format!("{e}")))?;
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
            reset_value: {
                if contents.peek(syn::Token![const]) {
                    contents.parse::<syn::Token![const]>()?;
                    contents.parse::<kw::RESET_VALUE>()?;
                    contents.parse::<syn::Token![:]>()?;
                    let t = contents.parse::<syn::Type>()?;
                    contents.parse::<syn::Token![=]>()?;
                    let v = contents.parse::<syn::Expr>()?;
                    contents.parse::<syn::Token![;]>()?;

                    parse_reset_value(t, v)?
                } else {
                    None
                }
            },
            description,
            fields: contents.parse_terminated(Field::parse, syn::Token![,])?,
        })
    }
}

fn parse_reset_value(t: syn::Type, v: Expr) -> Result<Option<ResetValue>, syn::Error> {
    Ok(match (t, v) {
        (
            syn::Type::Array(syn::TypeArray {
                elem,
                len:
                    Expr::Lit(ExprLit {
                        lit: Lit::Int(len), ..
                    }),
                ..
            }),
            syn::Expr::Array(syn::ExprArray { elems, .. }),
        ) => {
            if *elem != syn::parse_quote!(u8) {
                return Err(syn::Error::new(elem.span(), "Must be a u8 array"));
            }
            if len.base10_parse::<usize>()? != elems.len() {
                return Err(syn::Error::new(
                    elems.span(),
                    format!(
                        "Size of array ({}) does not correspond with the array type",
                        elems.len()
                    ),
                ));
            }

            let mut buffer = Vec::new();

            for elem in elems {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Int(elem),
                    ..
                }) = elem
                {
                    buffer.push(elem.base10_parse::<u8>()?);
                } else {
                    return Err(syn::Error::new(elem.span(), "Must be a u8 literal"));
                }
            }

            Some(ResetValue::new(buffer, true))
        }
        (
            syn::Type::Slice(syn::TypeSlice { elem, .. }),
            syn::Expr::Array(syn::ExprArray { elems, .. }),
        ) => {
            if *elem != syn::parse_quote!(u8) {
                return Err(syn::Error::new(elem.span(), "Must be a u8 array"));
            }

            let mut buffer = Vec::new();

            for elem in elems {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Int(elem),
                    ..
                }) = elem
                {
                    buffer.push(elem.base10_parse::<u8>()?);
                } else {
                    return Err(syn::Error::new(elem.span(), "Must be a u8 literal"));
                }
            }

            Some(ResetValue::new(buffer, true))
        }
        (
            syn::Type::Path(syn::TypePath { qself: None, path }),
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(value),
                ..
            }),
        ) => {
            if path == syn::parse_quote!(u8) {
                Some(ResetValue::new(
                    value.base10_parse::<u8>()?.to_be_bytes().into(),
                    false,
                ))
            } else if path == syn::parse_quote!(u16) {
                Some(ResetValue::new(
                    value.base10_parse::<u16>()?.to_be_bytes().into(),
                    false,
                ))
            } else if path == syn::parse_quote!(u32) {
                Some(ResetValue::new(
                    value.base10_parse::<u32>()?.to_be_bytes().into(),
                    false,
                ))
            } else if path == syn::parse_quote!(u64) {
                Some(ResetValue::new(
                    value.base10_parse::<u64>()?.to_be_bytes().into(),
                    false,
                ))
            } else if path == syn::parse_quote!(u128) {
                Some(ResetValue::new(
                    value.base10_parse::<u128>()?.to_be_bytes().into(),
                    false,
                ))
            } else {
                return Err(syn::Error::new(
                    path.span(),
                    "Must be a u8, u16, u32, u64 or u128",
                ));
            }
        }
        (t, _) => {
            return Err(syn::Error::new(t.span(), "Unsupported reset value type. Use `[u8; N]`, `[u8]` or an unsigned integer like `u16`"));
        }
    })
}

struct Field {
    name: syn::Ident,
    description: Option<String>,
    register_type: BaseType,
    conversion_type: ConversionType,
    bit_start: u32,
    bit_end: Option<u32>,
}

impl syn::parse::Parse for Field {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let field_attributes = syn::Attribute::parse_outer(input)?;

        let description = doc_string_from_attrs(&field_attributes)?;

        Ok(Self {
            name: input.parse()?,
            description,
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
            bit_end: if input.peek(syn::Token![..]) {
                input.parse::<syn::Token![..]>()?;
                Some(input.parse::<syn::LitInt>()?.base10_parse()?)
            } else {
                None
            },
        })
    }
}

enum ConversionType {
    None,
    Existing(syn::Path),
    Enum(Vec<(String, EnumVariant)>),
}

impl syn::parse::Parse for ConversionType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.parse::<syn::Token![as]>().is_ok() {
            if input.peek(syn::Token![enum]) {
                let item_enum = input.parse::<syn::ItemEnum>()?;

                let mut variants = Vec::new();

                for variant in item_enum.variants {
                    variants.push((variant.ident.to_string(), {
                        EnumVariant {
                            description: doc_string_from_attrs(&variant.attrs)?,
                            value: variant
                                .discriminant
                                .as_ref()
                                .map(|d| match &d.1 {
                                    syn::Expr::Lit(syn::ExprLit {
                                        lit: syn::Lit::Int(lit_int),
                                        ..
                                    }) => Ok(EnumVariantValue::Specified(
                                        lit_int.base10_parse().unwrap(),
                                    )),
                                    syn::Expr::Lit(syn::ExprLit {
                                        lit: syn::Lit::Str(lit_str),
                                        ..
                                    }) => match lit_str.value().as_str().try_into() {
                                        Ok(val) => Ok(val),
                                        Err(e) => Err(syn::Error::new(lit_str.span(), e)),
                                    },
                                    d => Err(syn::Error::new(
                                        d.span(),
                                        "Value not recognized. Must be a number or a string",
                                    )),
                                })
                                .transpose()?
                                .unwrap_or_default(),
                        }
                    }))
                }

                Ok(Self::Enum(variants))
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
    syn::custom_keyword!(RWType);
    syn::custom_keyword!(ADDRESS);
    syn::custom_keyword!(SIZE_BITS);
    syn::custom_keyword!(RESET_VALUE);
}

pub fn implement_device(item: TokenStream) -> TokenStream {
    let device_impl = match syn::parse2::<DeviceImpl>(item) {
        Ok(device_impl) => device_impl,
        Err(e) => return e.into_compile_error(),
    };

    let register_address_type = match device_impl
        .registers
        .first()
        .map(|r| r.address_type.to_string().as_str().try_into())
        .transpose()
    {
        Ok(Some(address_type)) => Some(address_type),
        Ok(None) => None,
        Err(e) => {
            return syn::Error::new(device_impl.registers[0].address_type.span(), format!("{e}"))
                .into_compile_error();
        }
    };

    let registers: RegisterCollection = device_impl
        .registers
        .into_iter()
        .map(|r| device_driver_generation::Register {
            name: r.name.to_string(),
            rw_type: r.rw_type,
            address: r.address_value,
            size_bits: r.size_bits_value,
            description: r.description,
            reset_value: r.reset_value,
            fields: r
                .fields
                .into_iter()
                .map(|f| device_driver_generation::Field {
                    name: f.name.to_string(),
                    description: f.description,
                    register_type: f.register_type,
                    conversion_type: match f.conversion_type {
                        ConversionType::None => None,
                        ConversionType::Existing(path) => {
                            Some(device_driver_generation::TypePathOrEnum::TypePath(
                                TypePath(path.to_token_stream().to_string()),
                            ))
                        }
                        ConversionType::Enum(enum_def) => {
                            Some(device_driver_generation::TypePathOrEnum::Enum(
                                FromIterator::from_iter(enum_def),
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
        .into();

    let registers = if registers.is_empty() {
        None
    } else {
        Some(registers)
    };

    let device = device_driver_generation::Device {
        register_address_type,
        registers,
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

fn doc_string_from_attrs(attrs: &[Attribute]) -> Result<Option<String>, syn::Error> {
    let mut description = String::new();

    for attr in attrs {
        let name_value = attr.meta.require_name_value()?;
        match (
            name_value.path.require_ident()?.to_string().as_str(),
            &name_value.value,
        ) {
            (
                "doc",
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(value),
                    ..
                }),
            ) => {
                description += &value.value();
            }
            (other, _) => {
                return Err(syn::Error::new(
                    name_value.path.span(),
                    format!("Attribute type `{other}` not supported in this usecase"),
                ));
            }
        }
    }

    let description = if description.is_empty() {
        None
    } else {
        Some(description)
    };

    Ok(description)
}
