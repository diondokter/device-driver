use device_driver_generation::{
    deserialization::{BufferCollection, CommandCollection, RegisterCollection},
    BaseType, ByteOrder, EnumVariant, EnumVariantValue, RWType, RegisterRepeat, ResetValue,
    TypePath,
};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{braced, punctuated::Punctuated, spanned::Spanned, Expr, ExprLit, Generics, Lit};

struct DeviceImpl {
    attrs: Vec<syn::Attribute>,
    impl_generics: syn::Generics,
    device_type: syn::Type,

    items: Punctuated<Item, syn::Token![,]>,
}

impl syn::parse::Parse for DeviceImpl {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = syn::Attribute::parse_outer(input)?;
        input.parse::<syn::Token![impl]>()?;

        let mut impl_generics: Generics = input.parse()?;

        let device_ident = input.parse()?;

        input.parse::<syn::AngleBracketedGenericArguments>().ok();

        impl_generics.where_clause = input.parse().ok();

        let items;
        braced!(items in input);

        let s = Self {
            attrs,
            impl_generics,
            device_type: device_ident,
            items: items.parse_terminated(Item::parse, syn::Token![,])?,
        };

        // Make sure all registers use the same address type
        {
            let mut registers = s
                .items
                .iter()
                .filter_map(Item::as_register)
                .flat_map(|r| r.address_types());
            if let Some(address_type) = registers.next() {
                for other_address_type in registers {
                    if *other_address_type != *address_type {
                        return Err(syn::Error::new(
                        other_address_type.span(),
                        format!("All registers must have the same address type. Previous type was `{}` and this is `{}`", address_type, other_address_type),
                    ));
                    }
                }
            }
        }

        Ok(s)
    }
}

enum Item {
    Register(Register),
    Command(Command),
    Buffer(Buffer),
}

impl Item {
    fn as_register(&self) -> Option<&Register> {
        if let Self::Register(v) = self {
            Some(v)
        } else {
            None
        }
    }

    fn as_command(&self) -> Option<&Command> {
        if let Self::Command(v) = self {
            Some(v)
        } else {
            None
        }
    }

    fn as_buffer(&self) -> Option<&Buffer> {
        if let Self::Buffer(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl syn::parse::Parse for Item {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attributes = syn::Attribute::parse_outer(input)?;

        if input.peek(kw::register) {
            Ok(Self::Register(Register::parse_register(input, attributes)?))
        } else if input.peek(kw::block) {
            Ok(Self::Register(Register::parse_block(input, attributes)?))
        } else if input.peek(syn::Token![ref]) {
            Ok(Self::Register(Register::parse_ref(input, attributes)?))
        } else if input.peek(kw::command) {
            Ok(Self::Command(Command::parse(input, attributes)?))
        } else if input.peek(kw::buffer) {
            Ok(Self::Buffer(Buffer::parse(input, attributes)?))
        } else {
            Err(syn::Error::new(
                input.span(),
                "Must be `register`, `command` or `buffer`",
            ))
        }
    }
}

struct Register {
    name: syn::Ident,
    description: Option<String>,
    cfg_attributes: Vec<syn::Attribute>,
    kind: RegisterKind,
}

impl Register {
    fn address_types(&self) -> Box<dyn Iterator<Item = &'_ syn::Ident> + '_> {
        match &self.kind {
            RegisterKind::Register { address_type, .. }
            | RegisterKind::Ref { address_type, .. } => Box::new(std::iter::once(address_type)),
            RegisterKind::Block {
                address_type,
                registers,
                ..
            } => Box::new(
                address_type
                    .as_ref()
                    .into_iter()
                    .chain(registers.iter().flat_map(|x| x.address_types())),
            ),
        }
    }

    fn parse_register(
        input: syn::parse::ParseStream,
        attributes: Vec<syn::Attribute>,
    ) -> syn::Result<Self> {
        let (description, cfg_attributes) = doc_string_and_cfg_from_attrs(&attributes)?;

        input.parse::<kw::register>()?;

        let name = input.parse()?;

        let contents;
        braced!(contents in input);

        Ok(Self {
            name,
            description,
            cfg_attributes,
            kind: RegisterKind::Register {
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
                byte_order: {
                    if contents.peek(syn::Token![type]) && contents.peek2(kw::ByteOrder) {
                        contents.parse::<syn::Token![type]>()?;
                        contents.parse::<kw::ByteOrder>()?;
                        contents.parse::<syn::Token![=]>()?;
                        let byte_order_value_ident = contents.parse::<syn::Ident>()?;
                        let value = byte_order_value_ident
                            .to_string()
                            .as_str()
                            .try_into()
                            .map_err(|e| {
                                syn::Error::new(byte_order_value_ident.span(), format!("{e}"))
                            })?;
                        contents.parse::<syn::Token![;]>()?;
                        Some(value)
                    } else {
                        None
                    }
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
                fields: contents
                    .parse_terminated(<Field as syn::parse::Parse>::parse, syn::Token![,])?,
            },
        })
    }

    fn parse_block(
        input: syn::parse::ParseStream,
        attributes: Vec<syn::Attribute>,
    ) -> syn::Result<Self> {
        let (description, cfg_attributes) = doc_string_and_cfg_from_attrs(&attributes)?;

        input.parse::<kw::block>()?;

        let name = input.parse()?;

        let contents;
        braced!(contents in input);

        let (address_type, base_address) =
            if contents.peek(syn::Token![const]) && contents.peek2(kw::BASE_ADDRESS) {
                let address_type = {
                    contents.parse::<syn::Token![const]>()?;
                    contents.parse::<kw::BASE_ADDRESS>()?;
                    contents.parse::<syn::Token![:]>()?;
                    contents.parse()?
                };
                contents.parse::<syn::Token![=]>()?;
                let base_address = contents.parse::<syn::LitInt>()?.base10_parse()?;
                contents.parse::<syn::Token![;]>()?;
                (Some(address_type), Some(base_address))
            } else {
                (None, None)
            };

        Ok(Self {
            name,
            description,
            cfg_attributes,
            kind: RegisterKind::Block {
                base_address,
                address_type,
                repeat: {
                    if contents.peek(syn::Token![const]) {
                        contents.parse::<syn::Token![const]>()?;
                        contents.parse::<kw::REPEAT>()?;
                        contents.parse::<syn::Token![=]>()?;

                        let repeat;
                        braced!(repeat in contents);

                        repeat.parse::<kw::count>()?;
                        repeat.parse::<syn::Token![:]>()?;
                        let count = repeat.parse::<syn::LitInt>()?.base10_parse()?;
                        repeat.parse::<syn::Token![,]>()?;

                        repeat.parse::<kw::stride>()?;
                        repeat.parse::<syn::Token![:]>()?;
                        let stride = repeat.parse::<syn::LitInt>()?.base10_parse()?;
                        if repeat.peek(syn::Token![,]) {
                            repeat.parse::<syn::Token![,]>()?;
                        }

                        Some(RegisterRepeat { count, stride })
                    } else {
                        None
                    }
                },
                registers: contents
                    .parse_terminated(<Register as syn::parse::Parse>::parse, syn::Token![,])?,
            },
        })
    }

    fn parse_ref(
        input: syn::parse::ParseStream,
        attributes: Vec<syn::Attribute>,
    ) -> syn::Result<Self> {
        let (description, cfg_attributes) = doc_string_and_cfg_from_attrs(&attributes)?;

        input.parse::<syn::Token![ref]>()?;

        let name = input.parse()?;

        input.parse::<syn::Token![=]>()?;
        let copy_of = input.parse()?;

        let contents;
        braced!(contents in input);

        Ok(Self {
            name,
            description,
            cfg_attributes,
            kind: RegisterKind::Ref {
                address_type: {
                    contents.parse::<syn::Token![const]>()?;
                    contents.parse::<kw::BASE_ADDRESS>()?;
                    contents.parse::<syn::Token![:]>()?;
                    contents.parse()?
                },
                base_address: {
                    contents.parse::<syn::Token![=]>()?;
                    let value = contents.parse::<syn::LitInt>()?.base10_parse()?;
                    contents.parse::<syn::Token![;]>()?;
                    value
                },
                copy_of,
            },
        })
    }
}

impl syn::parse::Parse for Register {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attributes = syn::Attribute::parse_outer(input)?;

        if input.peek(kw::register) {
            Ok(Register::parse_register(input, attributes)?)
        } else if input.peek(kw::block) {
            Ok(Register::parse_block(input, attributes)?)
        } else if input.peek(syn::Token![ref]) {
            Ok(Register::parse_ref(input, attributes)?)
        } else {
            Err(syn::Error::new(
                input.span(),
                "Must be `register`, `command` or `buffer`",
            ))
        }
    }
}

impl<'a> From<&'a Register> for device_driver_generation::Register {
    fn from(r: &'a Register) -> Self {
        Self {
            name: r.name.to_string(),
            description: r.description.clone(),
            cfg_attributes: r.cfg_attributes.clone(),
            kind: (&r.kind).into(),
        }
    }
}

enum RegisterKind {
    Register {
        rw_type: RWType,
        address_type: syn::Ident,
        address_value: u64,
        size_bits_value: u64,
        byte_order: Option<ByteOrder>,
        reset_value: Option<ResetValue>,
        fields: Punctuated<Field, syn::Token![,]>,
    },
    Block {
        address_type: Option<syn::Ident>,
        base_address: Option<u64>,
        repeat: Option<RegisterRepeat>,
        registers: Punctuated<Register, syn::Token![,]>,
    },
    Ref {
        address_type: syn::Ident,
        base_address: u64,
        copy_of: syn::Ident,
    },
}

impl<'a> From<&'a RegisterKind> for device_driver_generation::RegisterKind {
    fn from(value: &'a RegisterKind) -> Self {
        match value {
            RegisterKind::Register {
                rw_type,
                address_value,
                size_bits_value,
                byte_order,
                reset_value,
                fields,
                ..
            } => device_driver_generation::RegisterKind::Register {
                address: *address_value,
                rw_type: *rw_type,
                size_bits: *size_bits_value,
                reset_value: reset_value.clone(),
                byte_order: *byte_order,
                fields: fields
                    .iter()
                    .cloned()
                    .map(device_driver_generation::Field::from)
                    .collect::<Vec<_>>()
                    .into(),
            },
            RegisterKind::Block {
                base_address,
                repeat,
                registers,
                ..
            } => device_driver_generation::RegisterKind::Block {
                base_address: *base_address,
                repeat: *repeat,
                registers: registers
                    .iter()
                    .map(device_driver_generation::Register::from)
                    .collect::<Vec<_>>()
                    .into(),
            },
            RegisterKind::Ref {
                base_address,
                copy_of,
                ..
            } => device_driver_generation::RegisterKind::Ref {
                base_address: *base_address,
                copy_of: copy_of.to_string(),
            },
        }
    }
}

struct Command {
    name: syn::Ident,
    raw_value: u32,
    description: Option<String>,
    cfg_attributes: Vec<syn::Attribute>,
}

impl Command {
    fn parse(input: syn::parse::ParseStream, attributes: Vec<syn::Attribute>) -> syn::Result<Self> {
        let (description, cfg_attributes) = doc_string_and_cfg_from_attrs(&attributes)?;
        input.parse::<kw::command>()?;

        Ok(Self {
            name: input.parse()?,
            raw_value: {
                input.parse::<syn::Token![=]>()?;
                input.parse::<syn::LitInt>()?.base10_parse()?
            },
            description,
            cfg_attributes,
        })
    }
}

struct Buffer {
    name: syn::Ident,
    raw_value: u32,
    description: Option<String>,
    cfg_attributes: Vec<syn::Attribute>,
    rw_type: RWType,
}

impl Buffer {
    fn parse(input: syn::parse::ParseStream, attributes: Vec<syn::Attribute>) -> syn::Result<Self> {
        let (description, cfg_attributes) = doc_string_and_cfg_from_attrs(&attributes)?;
        input.parse::<kw::buffer>()?;

        Ok(Self {
            name: input.parse()?,
            rw_type: {
                input.parse::<syn::Token![:]>()?;
                let rw_type_value_ident = input.parse::<syn::Ident>()?;
                rw_type_value_ident
                    .to_string()
                    .as_str()
                    .try_into()
                    .map_err(|e| syn::Error::new(rw_type_value_ident.span(), format!("{e}")))?
            },
            raw_value: {
                input.parse::<syn::Token![=]>()?;
                input.parse::<syn::LitInt>()?.base10_parse()?
            },
            description,
            cfg_attributes,
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

#[derive(Clone)]
struct Field {
    name: syn::Ident,
    description: Option<String>,
    cfg_attributes: Vec<syn::Attribute>,
    register_type: BaseType,
    conversion_type: ConversionType,
    bit_start: u32,
    bit_end: Option<u32>,
}

impl syn::parse::Parse for Field {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let field_attributes = syn::Attribute::parse_outer(input)?;

        let (description, cfg_attributes) = doc_string_and_cfg_from_attrs(&field_attributes)?;

        Ok(Self {
            name: input.parse()?,
            description,
            cfg_attributes,
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

impl From<Field> for device_driver_generation::Field {
    fn from(f: Field) -> Self {
        device_driver_generation::Field {
            name: f.name.to_string(),
            description: f.description,
            cfg_attributes: f.cfg_attributes,
            register_type: f.register_type,
            conversion: match &f.conversion_type {
                ConversionType::Existing {
                    path,
                    strict: false,
                } => Some(device_driver_generation::TypePathOrEnum::TypePath(
                    TypePath(path.to_token_stream().to_string()),
                )),
                ConversionType::Enum {
                    value: enum_def,
                    strict: false,
                } => Some(device_driver_generation::TypePathOrEnum::Enum(
                    FromIterator::from_iter(enum_def.clone()),
                )),
                _ => None,
            },
            strict_conversion: match f.conversion_type {
                ConversionType::Existing { path, strict: true } => {
                    Some(device_driver_generation::TypePathOrEnum::TypePath(
                        TypePath(path.to_token_stream().to_string()),
                    ))
                }
                ConversionType::Enum {
                    value: enum_def,
                    strict: true,
                } => Some(device_driver_generation::TypePathOrEnum::Enum(
                    FromIterator::from_iter(enum_def),
                )),
                _ => None,
            },
            start: f.bit_start,
            end: f.bit_end,
        }
    }
}

#[derive(Clone)]
enum ConversionType {
    None,
    Existing {
        path: syn::Path,
        strict: bool,
    },
    Enum {
        value: Vec<(String, EnumVariant)>,
        strict: bool,
    },
}

impl syn::parse::Parse for ConversionType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.parse::<syn::Token![as]>().is_ok() {
            let strict = input.parse::<kw::strict>().is_ok();

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

                Ok(Self::Enum {
                    value: variants,
                    strict,
                })
            } else {
                Ok(Self::Existing {
                    path: input.parse()?,
                    strict,
                })
            }
        } else {
            Ok(Self::None)
        }
    }
}

mod kw {
    syn::custom_keyword!(block);
    syn::custom_keyword!(register);
    syn::custom_keyword!(command);
    syn::custom_keyword!(buffer);
    syn::custom_keyword!(strict);
    syn::custom_keyword!(count);
    syn::custom_keyword!(stride);
    syn::custom_keyword!(RWType);
    syn::custom_keyword!(ByteOrder);
    syn::custom_keyword!(ADDRESS);
    syn::custom_keyword!(BASE_ADDRESS);
    syn::custom_keyword!(SIZE_BITS);
    syn::custom_keyword!(RESET_VALUE);
    syn::custom_keyword!(REPEAT);
}

pub fn implement_device(item: TokenStream) -> TokenStream {
    let device_impl = match syn::parse2::<DeviceImpl>(item) {
        Ok(device_impl) => device_impl,
        Err(e) => return e.into_compile_error(),
    };

    let register_address_type = match device_impl
        .items
        .iter()
        .filter_map(Item::as_register)
        .flat_map(|r| r.address_types())
        .next()
    {
        Some(address_type) => match address_type.to_string().as_str().try_into() {
            Ok(address_type) => Some(address_type),
            Err(e) => {
                return syn::Error::new(address_type.span(), format!("{e}")).into_compile_error()
            }
        },
        None => None,
    };

    let registers: RegisterCollection = device_impl
        .items
        .iter()
        .filter_map(Item::as_register)
        .map(device_driver_generation::Register::from)
        .collect::<Vec<_>>()
        .into();

    let registers = if registers.is_empty() {
        None
    } else {
        Some(registers)
    };

    let commands: CommandCollection = device_impl
        .items
        .iter()
        .filter_map(Item::as_command)
        .map(|r| device_driver_generation::Command {
            name: r.name.to_string(),
            id: r.raw_value,
            description: r.description.clone(),
            cfg_attributes: r.cfg_attributes.clone(),
        })
        .collect::<Vec<_>>()
        .into();

    let commands = if commands.is_empty() {
        None
    } else {
        Some(commands)
    };

    let buffers: BufferCollection = device_impl
        .items
        .iter()
        .filter_map(Item::as_buffer)
        .map(|r| device_driver_generation::Buffer {
            name: r.name.to_string(),
            id: r.raw_value,
            description: r.description.clone(),
            cfg_attributes: r.cfg_attributes.clone(),
            rw_type: r.rw_type,
        })
        .collect::<Vec<_>>()
        .into();

    let buffers = if buffers.is_empty() {
        None
    } else {
        Some(buffers)
    };

    let device = device_driver_generation::Device {
        register_address_type,
        registers,
        commands,
        buffers,
    };

    let item = syn::ItemImpl {
        attrs: device_impl.attrs,
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

fn doc_string_from_attrs(attrs: &[syn::Attribute]) -> Result<Option<String>, syn::Error> {
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

fn doc_string_and_cfg_from_attrs(
    attrs: &[syn::Attribute],
) -> Result<(Option<String>, Vec<syn::Attribute>), syn::Error> {
    let mut description = String::new();
    let mut cfg_attributes = Vec::new();

    for attr in attrs {
        let path = attr.meta.path().require_ident()?.to_string();
        match path.as_str() {
            "doc" => {
                let name_value = attr.meta.require_name_value()?;
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(value),
                    ..
                }) = &name_value.value
                {
                    description += &value.value();
                    continue;
                }
            }
            "cfg" => {
                cfg_attributes.push(attr.clone());
                continue;
            }
            _ => {}
        }
        return Err(syn::Error::new(
            attr.meta.path().span(),
            format!("Attribute type `{path}` not supported in this usecase"),
        ));
    }

    let description = if description.is_empty() {
        None
    } else {
        Some(description)
    };

    Ok((description, cfg_attributes))
}
