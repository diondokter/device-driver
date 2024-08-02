//! # DSL grammar
//!
//! Top-level item is _Device_.
//!
//! - '*' is used to signal 0 or more instances.
//! - '?' is used to signal 0 or 1 instances.
//! - '|' is used as an 'or'. One of the options in the chain can be used.
//! - '( )' is used to group things together.
//! - Any `keyword` or brackets in the grammer use backticks just like word 'keyword' on this line.
//!
//! This doesn't map perfectly on the YAML and JSON inputs, but they should be made as close as possible.
//!
//! _Device_:
//! > _GlobalConfigList_
//! > _ObjectList_
//!
//! _GlobalConfigList_:
//! > (`config` `{` _GlobalConfig_* `}`)?
//!
//! _GlobalConfig_:
//! > (`type` `DefaultRegisterAccess` `=` _Access_`;`)
//! > | (`type` `DefaultFieldAccess` `=` _Access_`;`)
//! > | (`type` `DefaultBufferAccess` `=` _Access_`;`)
//! > | (`type` `DefaultByteOrder` `=` _ByteOrder_`;`)
//! > | (`type` `DefaultBitOrder` `=` _BitOrder_`;`)
//! > | (`type` `NameCasing` `=` _NameCasing_`;`)
//!
//! _NameCasing_:
//! This specifies the input, not the output.
//! > `Varying`|`PascalCase`|`SnakeCase`|`ScreamingSnakeCase`|`CamelCase`
//!
//! _ObjectList_:
//! > (_Object_(`,` _Object_)*`,`?)?
//!
//! _Object_:
//! > _Block_
//! > | _Register_
//! > | _Command_
//! > | _Buffer_
//! > | _RefObject_
//!
//! _RefObject_:
//! An object that is a copy of another object. Any items in the object are overrides.
//! > `ref` _IDENTIFIER_ `=` _Object_
//!
//! _AttributeList_:
//! > _Attribute_*
//!
//! _Attribute_:
//! Used for documentation and conditional compilation
//! > (`#` `[` `doc` `=` _STRING_`]`)
//! > | (`#` `[` `cfg` `(` _ConfigurationPredicate_`)` `]`)
//!
//! _Block_:
//! > _AttributeList_
//! > `block` _IDENTIFIER_ `{` _BlockItemList_ _ObjectList_ `}`
//!
//! _BlockItemList_:
//! > _BlockItem_*
//!
//! _BlockItem_:
//! > (`const` `ADDRESS_OFFSET` `=` _INTEGER_`;`)
//! > | _Repeat_
//!
//! _Register_:
//! > _AttributeList_
//! > `register` _IDENTIFIER_ `{` _RegisterItemList_ _FieldList_ `}`
//!
//! _RegisterItemList_:
//! > _RegisterItem_*
//!
//! _RegisterItem_:
//! > (`type` `Access` `=` _Access_`;`)
//! > | (`type` `ByteOrder` `=` _ByteOrder_`;`)
//! > | (`type` `BitOrder` `=` _BitOrder_`;`)
//! > | (`const` `ADDRESS` `=` _INTEGER_`;`)
//! > | (`const` `SIZE_BITS` `=` _INTEGER_`;`)
//! > | (`const` `RESET_VALUE` `=` _INTEGER_ | _U8_ARRAY_`;`)
//! > | _Repeat_
//!
//! _Access_:
//! > (`ReadWrite`|`RW`)|(`ReadClear`|`RC`)|(`ReadOnly`|`RO`)|(`WriteOnly`|`WO`)|(`ClearOnly`|`CO`)
//!
//! _ByteOrder_:
//! > `LE`|`BE`
//!
//! _BitOrder_:
//! > `LSB0`|`MSB0`
//!
//! _FieldList_:
//! > (_Field_ (`,` _Field_)* `,`?)
//!
//! _Field_:
//! > _AttributeList_
//! > _IDENTIFIER_`:` _BaseType_ _FieldConversion_? `=` _Access_? _FieldAddress_
//!
//! _FieldConversion_:
//! > (`as` _TYPE_PATH_)
//! > | (`as` `enum` _IDENTIFIER_ `{` _EnumVariantList_`}`)
//!
//! _EnumVariantList_:
//! > _EnumVariant_(`,` _EnumVariant_)*`,`?
//!
//! _EnumVariant_:
//! > _AttributeList_
//! > _IDENTIFIER_ (`=` _EnumValue_)?
//!
//! _EnumValue_:
//! > _INTEGER_|`default`|`catch_all`
//!
//! _FieldAddress_:
//! > _INTEGER_
//! > | (_INTEGER_`..`_INTEGER_)
//! > | (_INTEGER_`..=`_INTEGER_)
//!
//! _BaseType_:
//! > `bool` | `uint` | `int`
//!
//! _Command_:
//! > _AttributeList_
//! > `command` _IDENTIFIER_ _CommandValue_
//!
//! _CommandValue_:
//! > (`=` _INTEGER_)
//! > | (`{` _CommandItemList_ (`in` `{` _FieldList_ `}` `,`?)? (`out` `{` _FieldList_ `}` `,`?)? `}`)
//!
//! _CommandItemList_:
//! > _CommandItem_*
//!
//! _CommandItem_:
//! Commands have data going in and out, so they need two separate data field types.
//! If no in fields, then no data is sent. If no out fields, then no data is returned.
//! > (`type` `ByteOrder` `=` _ByteOrder_`;`)
//! > | (`type` `BitOrder` `=` _BitOrder_`;`)
//! > | (`const` `ADDRESS` `=` _INTEGER_`;`)
//! > | (`const` `SIZE_BITS` `=` _INTEGER_`;`)
//! > | _Repeat_
//!
//! _Repeat_:
//! > `const` `REPEAT` `=` `{` `count` `:` _INTEGER_`,` `stride` `:` _INTEGER_`,`? `}` `;`
//!
//! _Buffer_:
//! > _AttributeList_
//! > `buffer` _IDENTIFIER_(`:` _Access_)? `=` _INTEGER_

use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    LitInt, Token,
};

pub struct Device {
    pub global_config_list: GlobalConfigList,
    pub object_list: ObjectList,
}

impl Parse for Device {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            global_config_list: input.parse()?,
            object_list: input.parse()?,
        })
    }
}

pub struct GlobalConfigList {
    pub configs: Vec<GlobalConfig>,
}

impl Parse for GlobalConfigList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if !input.peek(kw::config) {
            return Ok(Self {
                configs: Vec::new(),
            });
        }

        input.parse::<kw::config>()?;
        let config_input;
        braced!(config_input in input);

        let mut configs = Vec::new();

        while !config_input.is_empty() {
            configs.push(config_input.parse()?);
        }

        Ok(Self { configs })
    }
}

pub enum GlobalConfig {
    DefaultRegisterAccess(Access),
    DefaultFieldAccess(Access),
    DefaultBufferAccess(Access),
    DefaultByteOrder(ByteOrder),
    DefaultBitOrder(BitOrder),
    NameCasing(NameCasing),
}

impl Parse for GlobalConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![type]>()?;

        let lookahead = input.lookahead1();

        if lookahead.peek(kw::DefaultRegisterAccess) {
            input.parse::<kw::DefaultRegisterAccess>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            input.parse::<Token![;]>()?;
            Ok(Self::DefaultRegisterAccess(value))
        } else if lookahead.peek(kw::DefaultFieldAccess) {
            input.parse::<kw::DefaultFieldAccess>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            input.parse::<Token![;]>()?;
            Ok(Self::DefaultFieldAccess(value))
        } else if lookahead.peek(kw::DefaultBufferAccess) {
            input.parse::<kw::DefaultBufferAccess>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            input.parse::<Token![;]>()?;
            Ok(Self::DefaultBufferAccess(value))
        } else if lookahead.peek(kw::DefaultByteOrder) {
            input.parse::<kw::DefaultByteOrder>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            input.parse::<Token![;]>()?;
            Ok(Self::DefaultByteOrder(value))
        } else if lookahead.peek(kw::DefaultBitOrder) {
            input.parse::<kw::DefaultBitOrder>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            input.parse::<Token![;]>()?;
            Ok(Self::DefaultBitOrder(value))
        } else if lookahead.peek(kw::NameCasing) {
            input.parse::<kw::NameCasing>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            input.parse::<Token![;]>()?;
            Ok(Self::NameCasing(value))
        } else {
            Err(lookahead.error())
        }
    }
}

pub enum NameCasing {
    Varying,
    PascalCase,
    SnakeCase,
    ScreamingSnakeCase,
    CamelCase,
}

impl Parse for NameCasing {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::Varying) {
            input.parse::<kw::Varying>()?;
            Ok(Self::Varying)
        } else if lookahead.peek(kw::PascalCase) {
            input.parse::<kw::PascalCase>()?;
            Ok(Self::PascalCase)
        } else if lookahead.peek(kw::SnakeCase) {
            input.parse::<kw::SnakeCase>()?;
            Ok(Self::SnakeCase)
        } else if lookahead.peek(kw::ScreamingSnakeCase) {
            input.parse::<kw::ScreamingSnakeCase>()?;
            Ok(Self::ScreamingSnakeCase)
        } else if lookahead.peek(kw::CamelCase) {
            input.parse::<kw::CamelCase>()?;
            Ok(Self::CamelCase)
        } else {
            Err(lookahead.error())
        }
    }
}

pub struct ObjectList {
    pub objects: Vec<Object>,
}

impl Parse for ObjectList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let punctuated_objects = Punctuated::<Object, Token![,]>::parse_terminated(input)?;
        Ok(Self {
            objects: punctuated_objects.into_iter().collect(),
        })
    }
}

pub enum Object {
    Block(Block),
    Register(Register),
    Command(Command),
    Buffer(Buffer),
    Ref(RefObject),
}

impl Parse for Object {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::block) {
            Ok(Self::Block(input.parse()?))
        } else if lookahead.peek(kw::register) {
            Ok(Self::Register(input.parse()?))
        } else if lookahead.peek(kw::command) {
            Ok(Self::Command(input.parse()?))
        } else if lookahead.peek(kw::buffer) {
            Ok(Self::Buffer(input.parse()?))
        } else if lookahead.peek(Token![ref]) {
            Ok(Self::Ref(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

pub struct RefObject {
    pub identifier: syn::Ident,
    pub object: Box<Object>,
}

impl Parse for RefObject {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![ref]>()?;

        let identifier = input.parse()?;

        input.parse::<Token![=]>()?;

        let object = input.parse()?;

        Ok(Self { identifier, object })
    }
}

pub struct AttributeList {
    pub attributes: Vec<Attribute>,
}

impl Parse for AttributeList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attributes = syn::Attribute::parse_outer(input)?;

        Ok(Self {
            attributes: attributes
                .into_iter()
                .map(
                    |attr| match attr.path().require_ident()?.to_string().as_str() {
                        "doc" => match &attr.meta.require_name_value()?.value {
                            syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(value),
                                ..
                            }) => Ok(Attribute::Doc(value.value())),
                            _ => Err(syn::Error::new_spanned(
                                attr,
                                "Invalid doc attribute format",
                            )),
                        },
                        "cfg" => {
                            Ok(Attribute::Cfg(attr.meta.require_list()?.tokens.to_string()))
                        }
                        val => {
                            Err(syn::Error::new_spanned(
                                attr,
                                format!("Unsupported attribute '{val}'. Only `doc` and `cfg` attributes are allowed"),
                            ))
                        }
                    },
                )
                .collect::<Result<_, _>>()?,
        })
    }
}

pub enum Attribute {
    Doc(String),
    Cfg(String),
}

pub struct Block {
    pub attribute_list: AttributeList,
    pub identifier: syn::Ident,
    pub block_item_list: BlockItemList,
    pub object_list: ObjectList,
}

impl Parse for Block {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attribute_list = input.parse()?;
        input.parse::<kw::block>()?;
        let identifier = input.parse()?;

        let braced_input;
        braced!(braced_input in input);

        let block_item_list = braced_input.parse()?;
        let object_list = braced_input.parse()?;

        Ok(Self {
            attribute_list,
            identifier,
            block_item_list,
            object_list,
        })
    }
}

pub struct BlockItemList {
    pub block_items: Vec<BlockItem>,
}

impl Parse for BlockItemList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut block_items = Vec::new();

        while !input.is_empty() && input.peek(Token![const]) {
            block_items.push(input.parse()?);
        }

        Ok(Self { block_items })
    }
}

pub enum BlockItem {
    AddressOffset(LitInt),
    Repeat(Repeat),
}

impl Parse for BlockItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek2(kw::ADDRESS_OFFSET) {
            input.parse::<Token![const]>()?;
            input.parse::<kw::ADDRESS_OFFSET>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            input.parse::<Token![;]>()?;

            Ok(Self::AddressOffset(value))
        } else if input.peek2(kw::REPEAT) {
            Ok(Self::Repeat(input.parse()?))
        } else {
            Err(syn::Error::new(
                input.span(),
                "Invalid value. Must be an `ADDRESS_OFFSET` or `REPEAT`",
            ))
        }
    }
}

pub struct Register {
    pub attribute_list: AttributeList,
    pub identifier: syn::Ident,
    pub register_item_list: RegisterItemList,
    pub field_list: FieldList,
}

impl Parse for Register {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attribute_list = input.parse()?;
        input.parse::<kw::register>()?;
        let identifier = input.parse()?;

        let braced_input;
        braced!(braced_input in input);

        let register_item_list = braced_input.parse()?;
        let field_list = braced_input.parse()?;

        Ok(Self {
            attribute_list,
            identifier,
            register_item_list,
            field_list,
        })
    }
}

pub struct RegisterItemList {
    pub register_items: Vec<RegisterItem>,
}

impl Parse for RegisterItemList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut register_items = Vec::new();

        loop {
            if input.peek(Token![type]) {
                input.parse::<Token![type]>()?;

                let lookahead = input.lookahead1();

                if lookahead.peek(kw::Access) {
                    input.parse::<kw::Access>()?;
                    input.parse::<Token![=]>()?;
                    let value = input.parse()?;
                    input.parse::<Token![;]>()?;
                    register_items.push(RegisterItem::Access(value));
                } else if lookahead.peek(kw::ByteOrder) {
                    input.parse::<kw::ByteOrder>()?;
                    input.parse::<Token![=]>()?;
                    let value = input.parse()?;
                    input.parse::<Token![;]>()?;
                    register_items.push(RegisterItem::ByteOrder(value));
                } else if lookahead.peek(kw::BitOrder) {
                    input.parse::<kw::BitOrder>()?;
                    input.parse::<Token![=]>()?;
                    let value = input.parse()?;
                    input.parse::<Token![;]>()?;
                    register_items.push(RegisterItem::BitOrder(value));
                } else {
                    return Err(lookahead.error());
                }
            } else if input.peek(Token![const]) {
                input.parse::<Token![const]>()?;

                let lookahead = input.lookahead1();

                if lookahead.peek(kw::ADDRESS) {
                    input.parse::<kw::ADDRESS>()?;
                    input.parse::<Token![=]>()?;
                    let value = input.parse()?;
                    input.parse::<Token![;]>()?;
                    register_items.push(RegisterItem::Adress(value));
                } else if lookahead.peek(kw::SIZE_BITS) {
                    input.parse::<kw::SIZE_BITS>()?;
                    input.parse::<Token![=]>()?;
                    let value = input.parse()?;
                    input.parse::<Token![;]>()?;
                    register_items.push(RegisterItem::SizeBits(value));
                } else if lookahead.peek(kw::RESET_VALUE) {
                    input.parse::<kw::RESET_VALUE>()?;
                    input.parse::<Token![=]>()?;

                    let lookahead = input.lookahead1();
                    let value = if lookahead.peek(syn::LitInt) {
                        RegisterItem::ResetValueInt(input.parse()?)
                    } else if lookahead.peek(syn::token::Bracket) {
                        let bracket_input;
                        bracketed!(bracket_input in input);

                        let elems =
                            Punctuated::<syn::LitInt, Token![,]>::parse_terminated(&bracket_input)?;

                        let mut reset_data = Vec::new();

                        for elem in elems {
                            reset_data.push(elem.base10_parse()?);
                        }

                        RegisterItem::ResetValueArray(reset_data)
                    } else {
                        return Err(lookahead.error());
                    };
                    input.parse::<Token![;]>()?;
                    register_items.push(value);
                } else if lookahead.peek(kw::REPEAT) {
                    RegisterItem::Repeat(input.parse()?);
                } else {
                    return Err(lookahead.error());
                }
            } else {
                break;
            }
        }

        Ok(Self { register_items })
    }
}

pub enum RegisterItem {
    Access(Access),
    ByteOrder(ByteOrder),
    BitOrder(BitOrder),
    Adress(LitInt),
    SizeBits(LitInt),
    ResetValueInt(LitInt),
    ResetValueArray(Vec<u8>),
    Repeat(Repeat),
}

pub enum Access {
    RW,
    RC,
    RO,
    WO,
    CO,
}

impl Parse for Access {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::ReadWrite) {
            input.parse::<kw::ReadWrite>()?;
            Ok(Self::RW)
        } else if lookahead.peek(kw::RW) {
            input.parse::<kw::RW>()?;
            Ok(Self::RW)
        } else if lookahead.peek(kw::ReadClear) {
            input.parse::<kw::ReadClear>()?;
            Ok(Self::RW)
        } else if lookahead.peek(kw::RC) {
            input.parse::<kw::RC>()?;
            Ok(Self::RW)
        } else if lookahead.peek(kw::ReadOnly) {
            input.parse::<kw::ReadOnly>()?;
            Ok(Self::RW)
        } else if lookahead.peek(kw::RO) {
            input.parse::<kw::RO>()?;
            Ok(Self::RW)
        } else if lookahead.peek(kw::WriteOnly) {
            input.parse::<kw::WriteOnly>()?;
            Ok(Self::WO)
        } else if lookahead.peek(kw::WO) {
            input.parse::<kw::WO>()?;
            Ok(Self::WO)
        } else if lookahead.peek(kw::ClearOnly) {
            input.parse::<kw::ClearOnly>()?;
            Ok(Self::CO)
        } else if lookahead.peek(kw::CO) {
            input.parse::<kw::CO>()?;
            Ok(Self::CO)
        } else {
            Err(lookahead.error())
        }
    }
}

pub enum ByteOrder {
    LE,
    BE,
}

impl Parse for ByteOrder {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::LE) {
            input.parse::<kw::LE>()?;
            Ok(Self::LE)
        } else if lookahead.peek(kw::BE) {
            input.parse::<kw::BE>()?;
            Ok(Self::BE)
        } else {
            Err(lookahead.error())
        }
    }
}

pub enum BitOrder {
    LSB0,
    MSB0,
}

impl Parse for BitOrder {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::LSB0) {
            input.parse::<kw::LSB0>()?;
            Ok(Self::LSB0)
        } else if lookahead.peek(kw::MSB0) {
            input.parse::<kw::MSB0>()?;
            Ok(Self::MSB0)
        } else {
            Err(lookahead.error())
        }
    }
}

pub struct FieldList {
    pub fields: Vec<Field>,
}

impl Parse for FieldList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let punctuated_fields = Punctuated::<Field, Token![,]>::parse_terminated(input)?;

        Ok(Self {
            fields: punctuated_fields.into_iter().collect(),
        })
    }
}

pub struct Field {
    pub attribute_list: AttributeList,
    pub identifier: syn::Ident,
    pub base_type: BaseType,
    pub field_conversion: Option<FieldConversion>,
    pub access: Option<Access>,
    pub field_address: FieldAddress,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attribute_list = input.parse()?;
        let identifier = input.parse()?;
        input.parse::<Token![:]>()?;
        let base_type = input.parse()?;

        let field_conversion = if input.peek(Token![as]) {
            Some(input.parse()?)
        } else {
            None
        };

        input.parse::<Token![=]>()?;

        let access = input.parse::<Access>().ok();
        let field_address = input.parse()?;

        Ok(Self {
            attribute_list,
            identifier,
            base_type,
            field_conversion,
            access,
            field_address,
        })
    }
}

pub enum FieldConversion {
    Direct(syn::Path),
    Enum {
        identifier: syn::Ident,
        enum_variant_list: EnumVariantList,
    },
}

impl Parse for FieldConversion {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![as]>()?;

        if !input.peek(Token![enum]) {
            return Ok(Self::Direct(input.parse::<syn::Path>()?));
        }

        input.parse::<Token![enum]>()?;
        let identifier = input.parse()?;

        let braced_input;
        braced!(braced_input in input);

        let enum_variant_list = braced_input.parse()?;

        Ok(Self::Enum {
            identifier,
            enum_variant_list,
        })
    }
}

pub struct EnumVariantList {
    pub variants: Vec<EnumVariant>,
}

impl Parse for EnumVariantList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let variants = Punctuated::<EnumVariant, Token![,]>::parse_terminated(input)?;
        Ok(Self {
            variants: variants.into_iter().collect(),
        })
    }
}

pub struct EnumVariant {
    pub attribute_list: AttributeList,
    pub identifier: syn::Ident,
    pub enum_value: Option<EnumValue>,
}

impl Parse for EnumVariant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attribute_list = input.parse()?;
        let identifier = input.parse()?;

        let enum_value = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;

            if let Ok(specification) = input.parse::<LitInt>() {
                Some(EnumValue::Specified(specification))
            } else if input.parse::<kw::default>().is_ok() {
                Some(EnumValue::Default)
            } else if input.parse::<kw::catch_all>().is_ok() {
                Some(EnumValue::CatchAll)
            } else {
                return Err(syn::Error::new(input.span(), "Specifier not recognized. Must be an integer literal, `default` or `catch_all`"));
            }
        } else {
            None
        };

        Ok(Self {
            attribute_list,
            identifier,
            enum_value,
        })
    }
}

pub enum EnumValue {
    Specified(LitInt),
    Default,
    CatchAll,
}

pub enum FieldAddress {
    Integer(LitInt),
    Range { start: LitInt, end: LitInt },
    RangeInclusive { start: LitInt, end: LitInt },
}

impl Parse for FieldAddress {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let start = input.parse()?;

        if input.peek(Token![..=]) {
            input.parse::<Token![..=]>()?;
            let end = input.parse()?;
            Ok(Self::RangeInclusive { start, end })
        } else if input.peek(Token![..]) {
            input.parse::<Token![..]>()?;
            let end = input.parse()?;
            Ok(Self::RangeInclusive { start, end })
        } else {
            Ok(FieldAddress::Integer(start))
        }
    }
}

pub enum BaseType {
    Bool,
    Uint,
    Int,
}

impl Parse for BaseType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::bool) {
            input.parse::<kw::bool>()?;
            Ok(BaseType::Bool)
        } else if lookahead.peek(kw::uint) {
            input.parse::<kw::uint>()?;
            Ok(BaseType::Uint)
        } else if lookahead.peek(kw::int) {
            input.parse::<kw::int>()?;
            Ok(BaseType::Int)
        } else {
            Err(lookahead.error())
        }
    }
}

pub struct Command {
    pub attribute_list: AttributeList,
    pub identifier: syn::Ident,
    pub value: CommandValue,
}

impl Parse for Command {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attribute_list = input.parse()?;
        input.parse::<kw::command>()?;
        let identifier = input.parse()?;
        let value = input.parse()?;

        Ok(Self {
            attribute_list,
            identifier,
            value,
        })
    }
}

pub enum CommandValue {
    Basic(LitInt),
    Extended {
        command_item_list: CommandItemList,
        in_field_list: FieldList,
        out_field_list: FieldList,
    },
}

impl Parse for CommandValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.parse::<Token![=]>().is_ok() {
            return Ok(CommandValue::Basic(input.parse()?));
        }

        let braced_input;
        braced!(braced_input in input);

        let command_item_list = braced_input.parse()?;

        let in_field_list = if braced_input.parse::<Token![in]>().is_ok() {
            let braced_input;
            braced!(braced_input in input);

            braced_input.parse()?
        } else {
            FieldList { fields: Vec::new() }
        };

        let out_field_list = if braced_input.parse::<kw::out>().is_ok() {
            let braced_input;
            braced!(braced_input in input);

            braced_input.parse()?
        } else {
            FieldList { fields: Vec::new() }
        };

        if !braced_input.is_empty() {
            return Err(syn::Error::new(
                braced_input.span(),
                "Did not expect any more tokens",
            ));
        }

        Ok(Self::Extended {
            command_item_list,
            in_field_list,
            out_field_list,
        })
    }
}

pub struct CommandItemList {
    pub items: Vec<CommandItem>,
}

pub enum CommandItem {
    ByteOrder(ByteOrder),
    BitOrder(BitOrder),
    Adress(LitInt),
    SizeBits(LitInt),
    Repeat(Repeat),
}

impl Parse for CommandItemList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();

        loop {
            if input.peek(Token![type]) {
                input.parse::<Token![type]>()?;

                let lookahead = input.lookahead1();

                if lookahead.peek(kw::ByteOrder) {
                    input.parse::<kw::ByteOrder>()?;
                    input.parse::<Token![=]>()?;
                    let value = input.parse()?;
                    input.parse::<Token![;]>()?;
                    items.push(CommandItem::ByteOrder(value));
                } else if lookahead.peek(kw::BitOrder) {
                    input.parse::<kw::BitOrder>()?;
                    input.parse::<Token![=]>()?;
                    let value = input.parse()?;
                    input.parse::<Token![;]>()?;
                    items.push(CommandItem::BitOrder(value));
                } else {
                    return Err(lookahead.error());
                }
            } else if input.peek(Token![const]) {
                input.parse::<Token![const]>()?;

                let lookahead = input.lookahead1();

                if lookahead.peek(kw::ADDRESS) {
                    input.parse::<kw::ADDRESS>()?;
                    input.parse::<Token![=]>()?;
                    let value = input.parse()?;
                    input.parse::<Token![;]>()?;
                    items.push(CommandItem::Adress(value));
                } else if lookahead.peek(kw::SIZE_BITS) {
                    input.parse::<kw::SIZE_BITS>()?;
                    input.parse::<Token![=]>()?;
                    let value = input.parse()?;
                    input.parse::<Token![;]>()?;
                    items.push(CommandItem::SizeBits(value));
                } else if lookahead.peek(kw::REPEAT) {
                    CommandItem::Repeat(input.parse()?);
                } else {
                    return Err(lookahead.error());
                }
            } else {
                break;
            }
        }

        Ok(Self { items })
    }
}

pub struct Repeat {
    pub count: LitInt,
    pub stride: LitInt,
}

impl Parse for Repeat {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![const]>()?;
        input.parse::<kw::REPEAT>()?;
        input.parse::<Token![=]>()?;

        let braced_input;
        braced!(braced_input in input);

        braced_input.parse::<kw::count>()?;
        braced_input.parse::<Token![:]>()?;
        let count = braced_input.parse()?;
        braced_input.parse::<Token![,]>()?;

        braced_input.parse::<kw::stride>()?;
        braced_input.parse::<Token![:]>()?;
        let stride = braced_input.parse()?;
        if braced_input.peek(Token![,]) {
            braced_input.parse::<Token![,]>()?;
        }

        input.parse::<Token![;]>()?;

        Ok(Repeat { count, stride })
    }
}

pub struct Buffer {
    pub attribute_list: AttributeList,
    pub identifier: syn::Ident,
    pub access: Option<Access>,
    pub address: LitInt,
}

impl Parse for Buffer {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attribute_list = input.parse()?;
        input.parse::<kw::buffer>()?;
        let identifier = input.parse()?;

        let access = if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        input.parse::<Token![=]>()?;

        let address = input.parse()?;

        Ok(Self {
            attribute_list,
            identifier,
            access,
            address: address,
        })
    }
}

mod kw {
    syn::custom_keyword!(config);

    // Objects
    syn::custom_keyword!(block);
    syn::custom_keyword!(register);
    syn::custom_keyword!(command);
    syn::custom_keyword!(buffer);

    syn::custom_keyword!(ADDRESS);
    syn::custom_keyword!(ADDRESS_OFFSET);
    syn::custom_keyword!(SIZE_BITS);
    syn::custom_keyword!(RESET_VALUE);

    // Repeat
    syn::custom_keyword!(REPEAT);
    syn::custom_keyword!(count);
    syn::custom_keyword!(stride);

    // Global config items
    syn::custom_keyword!(DefaultRegisterAccess);
    syn::custom_keyword!(DefaultFieldAccess);
    syn::custom_keyword!(DefaultBufferAccess);
    syn::custom_keyword!(DefaultByteOrder);
    syn::custom_keyword!(DefaultBitOrder);
    syn::custom_keyword!(NameCasing);

    // NameCasing options
    syn::custom_keyword!(Varying);
    syn::custom_keyword!(PascalCase);
    syn::custom_keyword!(SnakeCase);
    syn::custom_keyword!(ScreamingSnakeCase);
    syn::custom_keyword!(CamelCase);

    // Access
    syn::custom_keyword!(Access);
    syn::custom_keyword!(RW);
    syn::custom_keyword!(ReadWrite);
    syn::custom_keyword!(RC);
    syn::custom_keyword!(ReadClear);
    syn::custom_keyword!(RO);
    syn::custom_keyword!(ReadOnly);
    syn::custom_keyword!(WO);
    syn::custom_keyword!(WriteOnly);
    syn::custom_keyword!(CO);
    syn::custom_keyword!(ClearOnly);

    // ByteOrder
    syn::custom_keyword!(ByteOrder);
    syn::custom_keyword!(LE);
    syn::custom_keyword!(BE);

    // BitOrder
    syn::custom_keyword!(BitOrder);
    syn::custom_keyword!(LSB0);
    syn::custom_keyword!(MSB0);

    // BaseType
    syn::custom_keyword!(bool);
    syn::custom_keyword!(uint);
    syn::custom_keyword!(int);

    // EnumValue
    syn::custom_keyword!(default);
    syn::custom_keyword!(catch_all);

    // CommandValue
    syn::custom_keyword!(out);
}
