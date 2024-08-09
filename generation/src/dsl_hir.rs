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
//! > | (`type` `NameCase` `=` _NameCase_`;`)
//!
//! _NameCase_:
//! This specifies the input, not the output. Only applies to object and field names.
//! > `Varying`|`Pascal`|`Snake`|`ScreamingSnake`|`Camel`|`Kebab`|`Cobol`
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
//! > _AttributeList_
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
//! > | (`const` _Repeat_)
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
//! > | (`const` _Repeat_)
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
//! > _IDENTIFIER_`:` _Access_? _BaseType_ _FieldConversion_? `=` _FieldAddress_
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
//! > `command` _IDENTIFIER_ _CommandValue_?
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
//! > | (`const` _Repeat_)
//!
//! _Repeat_:
//! > `REPEAT` `=` `{` `count` `:` _INTEGER_`,` `stride` `:` _INTEGER_`,`? `}` `;`
//!
//! _Buffer_:
//! > _AttributeList_
//! > `buffer` _IDENTIFIER_(`:` _Access_)? (`=` _INTEGER_)?

use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    LitInt, Token,
};

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalConfig {
    DefaultRegisterAccess(Access),
    DefaultFieldAccess(Access),
    DefaultBufferAccess(Access),
    DefaultByteOrder(ByteOrder),
    DefaultBitOrder(BitOrder),
    NameCase(NameCase),
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
        } else if lookahead.peek(kw::NameCase) {
            input.parse::<kw::NameCase>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            input.parse::<Token![;]>()?;
            Ok(Self::NameCase(value))
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NameCase {
    Varying,
    Pascal,
    Snake,
    ScreamingSnake,
    Camel,
    Kebab,
    Cobol,
}

impl Parse for NameCase {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::Varying) {
            input.parse::<kw::Varying>()?;
            Ok(Self::Varying)
        } else if lookahead.peek(kw::Pascal) {
            input.parse::<kw::Pascal>()?;
            Ok(Self::Pascal)
        } else if lookahead.peek(kw::Snake) {
            input.parse::<kw::Snake>()?;
            Ok(Self::Snake)
        } else if lookahead.peek(kw::ScreamingSnake) {
            input.parse::<kw::ScreamingSnake>()?;
            Ok(Self::ScreamingSnake)
        } else if lookahead.peek(kw::Camel) {
            input.parse::<kw::Camel>()?;
            Ok(Self::Camel)
        } else if lookahead.peek(kw::Kebab) {
            input.parse::<kw::Kebab>()?;
            Ok(Self::Kebab)
        } else if lookahead.peek(kw::Cobol) {
            input.parse::<kw::Cobol>()?;
            Ok(Self::Cobol)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeList {
    pub attributes: Vec<Attribute>,
}

impl AttributeList {
    pub fn new() -> Self {
        Self {
            attributes: Vec::new(),
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attribute {
    Doc(String),
    Cfg(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
            input.parse::<Token![const]>()?;
            Ok(Self::Repeat(input.parse()?))
        } else {
            Err(syn::Error::new(
                input.span(),
                "Invalid value. Must be an `ADDRESS_OFFSET` or `REPEAT`",
            ))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterItemList {
    pub register_items: Vec<RegisterItem>,
}

impl RegisterItemList {
    pub fn new() -> Self {
        Self {
            register_items: Vec::new(),
        }
    }
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
                    register_items.push(RegisterItem::Address(value));
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
                    register_items.push(RegisterItem::Repeat(input.parse()?));
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterItem {
    Access(Access),
    ByteOrder(ByteOrder),
    BitOrder(BitOrder),
    Address(LitInt),
    SizeBits(LitInt),
    ResetValueInt(LitInt),
    ResetValueArray(Vec<u8>),
    Repeat(Repeat),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            Ok(Self::RC)
        } else if lookahead.peek(kw::RC) {
            input.parse::<kw::RC>()?;
            Ok(Self::RC)
        } else if lookahead.peek(kw::ReadOnly) {
            input.parse::<kw::ReadOnly>()?;
            Ok(Self::RO)
        } else if lookahead.peek(kw::RO) {
            input.parse::<kw::RO>()?;
            Ok(Self::RO)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldList {
    pub fields: Vec<Field>,
}

impl FieldList {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }
}

impl Parse for FieldList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let punctuated_fields = Punctuated::<Field, Token![,]>::parse_terminated(input)?;

        Ok(Self {
            fields: punctuated_fields.into_iter().collect(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub attribute_list: AttributeList,
    pub identifier: syn::Ident,
    pub access: Option<Access>,
    pub base_type: BaseType,
    pub field_conversion: Option<FieldConversion>,
    pub field_address: FieldAddress,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attribute_list = input.parse()?;
        let identifier = input.parse()?;
        input.parse::<Token![:]>()?;
        let access = input.parse::<Access>().ok();
        let base_type = input.parse()?;

        let field_conversion = if input.peek(Token![as]) {
            Some(input.parse()?)
        } else {
            None
        };

        input.parse::<Token![=]>()?;

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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariant {
    pub attribute_list: AttributeList,
    pub identifier: syn::Ident,
    pub enum_value: Option<EnumValue>,
}

impl Parse for EnumVariant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attribute_list = input.parse()?;
        let identifier = input.parse()?;

        let enum_value = if input.parse::<Token![=]>().is_ok() {
            Some(input.parse()?)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumValue {
    Specified(LitInt),
    Default,
    CatchAll,
}

impl Parse for EnumValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(specification) = input.parse::<LitInt>() {
            Ok(Self::Specified(specification))
        } else if input.parse::<kw::default>().is_ok() {
            Ok(Self::Default)
        } else if input.parse::<kw::catch_all>().is_ok() {
            Ok(Self::CatchAll)
        } else {
            Err(syn::Error::new(
                input.span(),
                "Specifier not recognized. Must be an integer literal, `default` or `catch_all`",
            ))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
            Ok(Self::Range { start, end })
        } else {
            Ok(FieldAddress::Integer(start))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub attribute_list: AttributeList,
    pub identifier: syn::Ident,
    pub value: Option<CommandValue>,
}

impl Parse for Command {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attribute_list = input.parse()?;
        input.parse::<kw::command>()?;
        let identifier = input.parse()?;

        let value = if !input.is_empty() {
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            attribute_list,
            identifier,
            value,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
            let braced_input_inner;
            braced!(braced_input_inner in braced_input);

            braced_input_inner.parse()?
        } else {
            FieldList { fields: Vec::new() }
        };

        let _ = braced_input.parse::<Token![,]>();

        let out_field_list = if braced_input.parse::<kw::out>().is_ok() {
            let braced_input_inner;
            braced!(braced_input_inner in braced_input);

            braced_input_inner.parse()?
        } else {
            FieldList { fields: Vec::new() }
        };

        let _ = braced_input.parse::<Token![,]>();

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandItemList {
    pub items: Vec<CommandItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandItem {
    ByteOrder(ByteOrder),
    BitOrder(BitOrder),
    Address(LitInt),
    SizeBits(LitInt),
    Repeat(Repeat),
}

impl Parse for CommandItemList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();

        loop {
            if input.parse::<Token![type]>().is_ok() {
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
                    items.push(CommandItem::Address(value));
                } else if lookahead.peek(kw::SIZE_BITS) {
                    input.parse::<kw::SIZE_BITS>()?;
                    input.parse::<Token![=]>()?;
                    let value = input.parse()?;
                    input.parse::<Token![;]>()?;
                    items.push(CommandItem::SizeBits(value));
                } else if lookahead.peek(kw::REPEAT) {
                    items.push(CommandItem::Repeat(input.parse()?));
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repeat {
    pub count: LitInt,
    pub stride: LitInt,
}

impl Parse for Repeat {
    fn parse(input: ParseStream) -> syn::Result<Self> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffer {
    pub attribute_list: AttributeList,
    pub identifier: syn::Ident,
    pub access: Option<Access>,
    pub address: Option<LitInt>,
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

        let address = if input.parse::<Token![=]>().is_ok() {
            Some(input.parse()?)
        } else {
            None
        };

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
    syn::custom_keyword!(NameCase);

    // NameCase options
    syn::custom_keyword!(Varying);
    syn::custom_keyword!(Pascal);
    syn::custom_keyword!(Snake);
    syn::custom_keyword!(ScreamingSnake);
    syn::custom_keyword!(Camel);
    syn::custom_keyword!(Kebab);
    syn::custom_keyword!(Cobol);

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

#[cfg(test)]
mod tests {
    use proc_macro2::Span;
    use syn::Ident;

    use super::*;

    #[test]
    fn parse_access() {
        assert_eq!(syn::parse_str::<Access>("RW").unwrap(), Access::RW);
        assert_eq!(syn::parse_str::<Access>("ReadWrite").unwrap(), Access::RW);
        assert_eq!(syn::parse_str::<Access>("RC").unwrap(), Access::RC);
        assert_eq!(syn::parse_str::<Access>("ReadClear").unwrap(), Access::RC);
        assert_eq!(syn::parse_str::<Access>("RO").unwrap(), Access::RO);
        assert_eq!(syn::parse_str::<Access>("ReadOnly").unwrap(), Access::RO);
        assert_eq!(syn::parse_str::<Access>("WO").unwrap(), Access::WO);
        assert_eq!(syn::parse_str::<Access>("WriteOnly").unwrap(), Access::WO);
        assert_eq!(syn::parse_str::<Access>("CO").unwrap(), Access::CO);
        assert_eq!(syn::parse_str::<Access>("ClearOnly").unwrap(), Access::CO);

        assert_eq!(
            syn::parse_str::<Access>("ABCD").unwrap_err().to_string(),
            "expected one of: `ReadWrite`, `RW`, `ReadClear`, `RC`, `ReadOnly`, `RO`, `WriteOnly`, `WO`, `ClearOnly`, `CO`"
        );
    }

    #[test]
    fn parse_byte_order() {
        assert_eq!(syn::parse_str::<ByteOrder>("LE").unwrap(), ByteOrder::LE);
        assert_eq!(syn::parse_str::<ByteOrder>("BE").unwrap(), ByteOrder::BE);

        assert_eq!(
            syn::parse_str::<ByteOrder>("ABCD").unwrap_err().to_string(),
            "expected `LE` or `BE`"
        );
    }

    #[test]
    fn parse_bit_order() {
        assert_eq!(syn::parse_str::<BitOrder>("LSB0").unwrap(), BitOrder::LSB0);
        assert_eq!(syn::parse_str::<BitOrder>("MSB0").unwrap(), BitOrder::MSB0);

        assert_eq!(
            syn::parse_str::<BitOrder>("ABCD").unwrap_err().to_string(),
            "expected `LSB0` or `MSB0`"
        );
    }

    #[test]
    fn parse_base_type() {
        assert_eq!(syn::parse_str::<BaseType>("bool").unwrap(), BaseType::Bool);
        assert_eq!(syn::parse_str::<BaseType>("uint").unwrap(), BaseType::Uint);
        assert_eq!(syn::parse_str::<BaseType>("int").unwrap(), BaseType::Int);

        assert_eq!(
            syn::parse_str::<BaseType>("ABCD").unwrap_err().to_string(),
            "expected one of: `bool`, `uint`, `int`"
        );
    }

    #[test]
    fn parse_enum_value() {
        assert_eq!(
            syn::parse_str::<EnumValue>("55").unwrap(),
            EnumValue::Specified(LitInt::new("55", Span::call_site()))
        );
        assert_eq!(
            syn::parse_str::<EnumValue>("default").unwrap(),
            EnumValue::Default
        );
        assert_eq!(
            syn::parse_str::<EnumValue>("catch_all").unwrap(),
            EnumValue::CatchAll
        );

        assert_eq!(
            syn::parse_str::<EnumValue>("ABCD").unwrap_err().to_string(),
            "Specifier not recognized. Must be an integer literal, `default` or `catch_all`"
        );
    }

    #[test]
    fn parse_repeat() {
        assert_eq!(
            syn::parse_str::<Repeat>("REPEAT = { count: 55, stride: 0x123, };").unwrap(),
            Repeat {
                count: LitInt::new("55", Span::call_site()),
                stride: LitInt::new("0x123", Span::call_site())
            }
        );
        assert_eq!(
            syn::parse_str::<Repeat>("REPEAT = { count: 55, stride: 0x123 };").unwrap(),
            Repeat {
                count: LitInt::new("55", Span::call_site()),
                stride: LitInt::new("0x123", Span::call_site())
            }
        );

        assert_eq!(
            syn::parse_str::<Repeat>("ABCD").unwrap_err().to_string(),
            "expected `REPEAT`"
        );
        assert_eq!(
            syn::parse_str::<Repeat>("REPEAT = { count: 55 stride: 0x123 };")
                .unwrap_err()
                .to_string(),
            "expected `,`"
        );
        assert_eq!(
            syn::parse_str::<Repeat>("REPEAT = ")
                .unwrap_err()
                .to_string(),
            "unexpected end of input, expected curly braces"
        );
    }

    #[test]
    fn parse_command_item_list() {
        assert_eq!(
            syn::parse_str::<CommandItemList>("").unwrap(),
            CommandItemList { items: vec![] }
        );

        assert_eq!(
            syn::parse_str::<CommandItemList>("type ByteOrder = LE;").unwrap(),
            CommandItemList {
                items: vec![CommandItem::ByteOrder(ByteOrder::LE)]
            }
        );

        assert_eq!(
            syn::parse_str::<CommandItemList>("type BitOrder = LSB0;\nconst ADDRESS = 123;")
                .unwrap(),
            CommandItemList {
                items: vec![
                    CommandItem::BitOrder(BitOrder::LSB0),
                    CommandItem::Address(LitInt::new("123", Span::call_site()))
                ]
            }
        );

        assert_eq!(
            syn::parse_str::<CommandItemList>(
                "const SIZE_BITS = 16;\nconst REPEAT = { count: 2, stride: 2 };"
            )
            .unwrap(),
            CommandItemList {
                items: vec![
                    CommandItem::SizeBits(LitInt::new("16", Span::call_site())),
                    CommandItem::Repeat(Repeat {
                        count: LitInt::new("2", Span::call_site()),
                        stride: LitInt::new("2", Span::call_site())
                    })
                ]
            }
        );

        assert_eq!(
            syn::parse_str::<CommandItemList>("const ABC = 16;")
                .unwrap_err()
                .to_string(),
            "expected one of: `ADDRESS`, `SIZE_BITS`, `REPEAT`"
        );

        assert_eq!(
            syn::parse_str::<CommandItemList>("type ABC = 16;")
                .unwrap_err()
                .to_string(),
            "expected `ByteOrder` or `BitOrder`"
        );
    }

    #[test]
    fn parse_field_address() {
        assert_eq!(
            syn::parse_str::<FieldAddress>("55").unwrap(),
            FieldAddress::Integer(LitInt::new("55", Span::call_site()))
        );
        assert_eq!(
            syn::parse_str::<FieldAddress>("55..=0x123").unwrap(),
            FieldAddress::RangeInclusive {
                start: LitInt::new("55", Span::call_site()),
                end: LitInt::new("0x123", Span::call_site())
            }
        );
        assert_eq!(
            syn::parse_str::<FieldAddress>("55..0x123").unwrap(),
            FieldAddress::Range {
                start: LitInt::new("55", Span::call_site()),
                end: LitInt::new("0x123", Span::call_site())
            }
        );

        assert_eq!(
            syn::parse_str::<FieldAddress>("ABCD")
                .unwrap_err()
                .to_string(),
            "expected integer literal"
        );
    }

    #[test]
    fn parse_buffer() {
        assert_eq!(
            syn::parse_str::<Buffer>("buffer TestBuffer = 0x123").unwrap(),
            Buffer {
                attribute_list: AttributeList::new(),
                identifier: Ident::new("TestBuffer", Span::call_site()),
                access: None,
                address: Some(LitInt::new("0x123", Span::call_site())),
            }
        );

        assert_eq!(
            syn::parse_str::<Buffer>("buffer TestBuffer").unwrap(),
            Buffer {
                attribute_list: AttributeList::new(),
                identifier: Ident::new("TestBuffer", Span::call_site()),
                access: None,
                address: None,
            }
        );

        assert_eq!(
            syn::parse_str::<Buffer>("buffer TestBuffer: CO").unwrap(),
            Buffer {
                attribute_list: AttributeList::new(),
                identifier: Ident::new("TestBuffer", Span::call_site()),
                access: Some(Access::CO),
                address: None,
            }
        );

        assert_eq!(
            syn::parse_str::<Buffer>("buffer TestBuffer =")
                .unwrap_err()
                .to_string(),
            "unexpected end of input, expected integer literal"
        );

        assert_eq!(
            syn::parse_str::<Buffer>("/// A test buffer\nbuffer TestBuffer: RO = 0x123").unwrap(),
            Buffer {
                attribute_list: AttributeList {
                    attributes: vec![Attribute::Doc(" A test buffer".into())]
                },
                identifier: Ident::new("TestBuffer", Span::call_site()),
                access: Some(Access::RO),
                address: Some(LitInt::new("0x123", Span::call_site())),
            }
        );
    }

    #[test]
    fn parse_field() {
        assert_eq!(
            syn::parse_str::<Field>("TestField: ClearOnly int = 0x123").unwrap(),
            Field {
                attribute_list: AttributeList::new(),
                identifier: Ident::new("TestField".into(), Span::call_site()),
                access: Some(Access::CO),
                base_type: BaseType::Int,
                field_conversion: None,
                field_address: FieldAddress::Integer(LitInt::new("0x123", Span::call_site()))
            }
        );

        assert_eq!(
            syn::parse_str::<Field>("ExsitingType: RW uint as crate::module::foo::Bar = 0x1234")
                .unwrap(),
            Field {
                attribute_list: AttributeList::new(),
                identifier: Ident::new("ExsitingType".into(), Span::call_site()),
                access: Some(Access::RW),
                base_type: BaseType::Uint,
                field_conversion: Some(FieldConversion::Direct(
                    syn::parse_str("crate::module::foo::Bar").unwrap()
                )),
                field_address: FieldAddress::Integer(LitInt::new("0x1234", Span::call_site()))
            }
        );

        assert_eq!(
            syn::parse_str::<Field>(
                "ExsitingType: RW uint as enum crate::module::foo::Bar = 0x1234"
            )
            .unwrap_err()
            .to_string(),
            "expected identifier, found keyword `crate`"
        );

        assert_eq!(
            syn::parse_str::<Field>("ExsitingType: RW uint as enum Bar { } = 0x1234").unwrap(),
            Field {
                attribute_list: AttributeList::new(),
                identifier: Ident::new("ExsitingType".into(), Span::call_site()),
                access: Some(Access::RW),
                base_type: BaseType::Uint,
                field_conversion: Some(FieldConversion::Enum {
                    identifier: Ident::new("Bar", Span::call_site()),
                    enum_variant_list: EnumVariantList {
                        variants: Vec::new()
                    }
                }),
                field_address: FieldAddress::Integer(LitInt::new("0x1234", Span::call_site()))
            }
        );
    }

    #[test]
    fn parse_enum_variant_list() {
        assert_eq!(
            syn::parse_str::<EnumVariantList>(
                "A, B = 0xFF,\n/// This is C\nC = default, D = catch_all"
            )
            .unwrap(),
            EnumVariantList {
                variants: vec![
                    EnumVariant {
                        attribute_list: AttributeList::new(),
                        identifier: Ident::new("A", Span::call_site()),
                        enum_value: None
                    },
                    EnumVariant {
                        attribute_list: AttributeList::new(),
                        identifier: Ident::new("B", Span::call_site()),
                        enum_value: Some(EnumValue::Specified(LitInt::new(
                            "0xFF",
                            Span::call_site()
                        )))
                    },
                    EnumVariant {
                        attribute_list: AttributeList {
                            attributes: vec![Attribute::Doc(" This is C".into())]
                        },
                        identifier: Ident::new("C", Span::call_site()),
                        enum_value: Some(EnumValue::Default)
                    },
                    EnumVariant {
                        attribute_list: AttributeList::new(),
                        identifier: Ident::new("D", Span::call_site()),
                        enum_value: Some(EnumValue::CatchAll)
                    },
                ]
            }
        );
    }

    #[test]
    fn parse_command() {
        assert_eq!(
            syn::parse_str::<Command>("/// A command!\n#[cfg(feature = \"std\")]\ncommand Foo = 5")
                .unwrap(),
            Command {
                attribute_list: AttributeList {
                    attributes: vec![
                        Attribute::Doc(" A command!".into()),
                        Attribute::Cfg("feature = \"std\"".into()),
                    ]
                },
                identifier: Ident::new("Foo", Span::call_site()),
                value: Some(CommandValue::Basic(LitInt::new("5", Span::call_site()))),
            }
        );
        assert_eq!(
            syn::parse_str::<Command>("command Bar { type BitOrder = LSB0; }").unwrap(),
            Command {
                attribute_list: AttributeList::new(),
                identifier: Ident::new("Bar", Span::call_site()),
                value: Some(CommandValue::Extended {
                    command_item_list: CommandItemList {
                        items: vec![CommandItem::BitOrder(BitOrder::LSB0)]
                    },
                    in_field_list: FieldList { fields: vec![] },
                    out_field_list: FieldList { fields: vec![] }
                }),
            }
        );

        assert_eq!(
            syn::parse_str::<Command>("command Bar { in { } }").unwrap(),
            Command {
                attribute_list: AttributeList::new(),
                identifier: Ident::new("Bar", Span::call_site()),
                value: Some(CommandValue::Extended {
                    command_item_list: CommandItemList { items: vec![] },
                    in_field_list: FieldList { fields: vec![] },
                    out_field_list: FieldList { fields: vec![] }
                }),
            }
        );

        assert_eq!(
            syn::parse_str::<Command>("command Bar { in { }, out { }, }").unwrap(),
            Command {
                attribute_list: AttributeList::new(),
                identifier: Ident::new("Bar", Span::call_site()),
                value: Some(CommandValue::Extended {
                    command_item_list: CommandItemList { items: vec![] },
                    in_field_list: FieldList { fields: vec![] },
                    out_field_list: FieldList { fields: vec![] }
                }),
            }
        );

        assert_eq!(
            syn::parse_str::<Command>("command Bar { out { foo: bool = 0 } }").unwrap(),
            Command {
                attribute_list: AttributeList::new(),
                identifier: Ident::new("Bar", Span::call_site()),
                value: Some(CommandValue::Extended {
                    command_item_list: CommandItemList { items: vec![] },
                    in_field_list: FieldList { fields: vec![] },
                    out_field_list: FieldList {
                        fields: vec![Field {
                            attribute_list: AttributeList::new(),
                            identifier: Ident::new("foo", Span::call_site()),
                            access: None,
                            base_type: BaseType::Bool,
                            field_conversion: None,
                            field_address: FieldAddress::Integer(LitInt::new(
                                "0",
                                Span::call_site()
                            ))
                        }]
                    }
                }),
            }
        );

        assert_eq!(
            syn::parse_str::<Command>("command Bar { in { }, out { }, more stuff! }")
                .unwrap_err()
                .to_string(),
            "Did not expect any more tokens"
        );

        assert_eq!(
            syn::parse_str::<Command>("command Bar").unwrap(),
            Command {
                attribute_list: AttributeList::new(),
                identifier: Ident::new("Bar", Span::call_site()),
                value: None,
            }
        );
    }

    #[test]
    fn parse_register_item_list() {
        assert_eq!(
            syn::parse_str::<RegisterItemList>("").unwrap(),
            RegisterItemList {
                register_items: vec![]
            }
        );

        assert_eq!(
            syn::parse_str::<RegisterItemList>("type Access = RW;").unwrap(),
            RegisterItemList {
                register_items: vec![RegisterItem::Access(Access::RW)]
            }
        );

        assert_eq!(
            syn::parse_str::<RegisterItemList>("type Access = RW")
                .unwrap_err()
                .to_string(),
            "expected `;`"
        );

        assert_eq!(
            syn::parse_str::<RegisterItemList>("type ByteOrder = LE;\ntype BitOrder = LSB0;")
                .unwrap(),
            RegisterItemList {
                register_items: vec![
                    RegisterItem::ByteOrder(ByteOrder::LE),
                    RegisterItem::BitOrder(BitOrder::LSB0)
                ]
            }
        );

        assert_eq!(
            syn::parse_str::<RegisterItemList>("const RST_VALUE = 5;")
                .unwrap_err()
                .to_string(),
            "expected one of: `ADDRESS`, `SIZE_BITS`, `RESET_VALUE`, `REPEAT`"
        );

        assert_eq!(
            syn::parse_str::<RegisterItemList>("type BT_ORDR = LSB0;")
                .unwrap_err()
                .to_string(),
            "expected one of: `Access`, `ByteOrder`, `BitOrder`"
        );

        assert_eq!(
            syn::parse_str::<RegisterItemList>(
                "const ADDRESS = 0x123;\nconst SIZE_BITS = 16;\nconst RESET_VALUE = 0xFFFF;"
            )
            .unwrap(),
            RegisterItemList {
                register_items: vec![
                    RegisterItem::Address(LitInt::new("0x123", Span::call_site())),
                    RegisterItem::SizeBits(LitInt::new("16", Span::call_site())),
                    RegisterItem::ResetValueInt(LitInt::new("0xFFFF", Span::call_site()))
                ]
            }
        );

        assert_eq!(
            syn::parse_str::<RegisterItemList>("const RESET_VALUE = [0, 1, 2, 0x30];").unwrap(),
            RegisterItemList {
                register_items: vec![RegisterItem::ResetValueArray(vec![0, 1, 2, 0x30])]
            }
        );

        assert_eq!(
            syn::parse_str::<RegisterItemList>("const RESET_VALUE = [0, 1, 2, 0x300];")
                .unwrap_err()
                .to_string(),
            "number too large to fit in target type"
        );

        assert_eq!(
            syn::parse_str::<RegisterItemList>("const REPEAT = { count: 0, stride: 0 };").unwrap(),
            RegisterItemList {
                register_items: vec![RegisterItem::Repeat(Repeat {
                    count: LitInt::new("0", Span::call_site()),
                    stride: LitInt::new("0", Span::call_site())
                })]
            }
        );

        assert_eq!(
            syn::parse_str::<RegisterItemList>("const RRRRRESET_VALUE = [0, 1, 2, 0x30];")
                .unwrap_err()
                .to_string(),
            "expected one of: `ADDRESS`, `SIZE_BITS`, `RESET_VALUE`, `REPEAT`"
        );

        assert_eq!(
            syn::parse_str::<RegisterItemList>("const RESET_VALUE = ;")
                .unwrap_err()
                .to_string(),
            "expected integer literal or square brackets"
        );
    }

    #[test]
    fn parse_attribute_list() {
        assert_eq!(
            syn::parse_str::<AttributeList>("#[custom]")
                .unwrap_err()
                .to_string(),
            "Unsupported attribute 'custom'. Only `doc` and `cfg` attributes are allowed"
        );
        assert_eq!(
            syn::parse_str::<AttributeList>("#[doc(bla)]")
                .unwrap_err()
                .to_string(),
            "expected `=`"
        );
        assert_eq!(
            syn::parse_str::<AttributeList>("#[doc = 1]")
                .unwrap_err()
                .to_string(),
            "Invalid doc attribute format"
        );
    }

    #[test]
    fn parse_ref_object() {
        assert_eq!(
            syn::parse_str::<RefObject>("ref MyRef = command MyOriginal").unwrap(),
            RefObject {
                identifier: Ident::new("MyRef", Span::call_site()),
                object: Box::new(Object::Command(Command {
                    attribute_list: AttributeList::new(),
                    identifier: Ident::new("MyOriginal", Span::call_site()),
                    value: None
                }))
            }
        );
    }

    #[test]
    fn parse_register() {
        assert_eq!(
            syn::parse_str::<Register>("register Foo { }").unwrap(),
            Register {
                attribute_list: AttributeList::new(),
                identifier: Ident::new("Foo", Span::call_site()),
                field_list: FieldList::new(),
                register_item_list: RegisterItemList::new(),
            }
        );

        assert_eq!(
            syn::parse_str::<Register>("register Foo")
                .unwrap_err()
                .to_string(),
            "unexpected end of input, expected curly braces"
        );

        assert_eq!(
            syn::parse_str::<Register>(
                "/// Hello!\nregister Foo { type Access = RW; TestField: ClearOnly int = 0x123, }"
            )
            .unwrap(),
            Register {
                attribute_list: AttributeList {
                    attributes: vec![Attribute::Doc(" Hello!".into())]
                },
                identifier: Ident::new("Foo", Span::call_site()),
                register_item_list: RegisterItemList {
                    register_items: vec![RegisterItem::Access(Access::RW)]
                },
                field_list: FieldList {
                    fields: vec![Field {
                        attribute_list: AttributeList::new(),
                        identifier: Ident::new("TestField".into(), Span::call_site()),
                        access: Some(Access::CO),
                        base_type: BaseType::Int,
                        field_conversion: None,
                        field_address: FieldAddress::Integer(LitInt::new(
                            "0x123",
                            Span::call_site()
                        ))
                    }]
                },
            }
        );
    }

    #[test]
    fn parse_block_item_list() {
        assert_eq!(
            syn::parse_str::<BlockItemList>("").unwrap(),
            BlockItemList {
                block_items: vec![]
            }
        );

        assert_eq!(
            syn::parse_str::<BlockItemList>("const ADDRESS_OFFSET = 2;").unwrap(),
            BlockItemList {
                block_items: vec![BlockItem::AddressOffset(LitInt::new(
                    "2",
                    Span::call_site()
                ))]
            }
        );

        assert_eq!(
            syn::parse_str::<BlockItemList>(
                "const ADDRESS_OFFSET = 2; const REPEAT = { count: 0, stride: 0 };"
            )
            .unwrap(),
            BlockItemList {
                block_items: vec![
                    BlockItem::AddressOffset(LitInt::new("2", Span::call_site())),
                    BlockItem::Repeat(Repeat {
                        count: LitInt::new("0", Span::call_site()),
                        stride: LitInt::new("0", Span::call_site())
                    })
                ]
            }
        );

        assert_eq!(
            syn::parse_str::<BlockItemList>("const ADDRESS = 2;")
                .unwrap_err()
                .to_string(),
            "Invalid value. Must be an `ADDRESS_OFFSET` or `REPEAT`"
        );
    }

    #[test]
    fn parse_block() {
        assert_eq!(
            syn::parse_str::<Block>("block MyBlock {}").unwrap(),
            Block {
                attribute_list: AttributeList::new(),
                identifier: Ident::new("MyBlock", Span::call_site()),
                block_item_list: BlockItemList {
                    block_items: vec![]
                },
                object_list: ObjectList { objects: vec![] },
            }
        );

        assert_eq!(
            syn::parse_str::<Block>("/// Hi there\nblock MyBlock { const ADDRESS_OFFSET = 5; command A = 5, buffer B = 6 }").unwrap(),
            Block {
                attribute_list: AttributeList { attributes: vec![Attribute::Doc(" Hi there".into())] },
                identifier: Ident::new("MyBlock", Span::call_site()),
                block_item_list: BlockItemList {
                    block_items: vec![BlockItem::AddressOffset(LitInt::new("5", Span::call_site()))]
                },
                object_list: ObjectList {
                    objects: vec![
                        Object::Command(Command {
                            attribute_list: AttributeList::new(),
                            identifier: Ident::new("A", Span::call_site()),
                            value: Some(CommandValue::Basic(LitInt::new("5", Span::call_site())))
                        }),
                        Object::Buffer(Buffer {
                            attribute_list: AttributeList::new(),
                            identifier: Ident::new("B", Span::call_site()),
                            access: None,
                            address: Some(LitInt::new("6", Span::call_site()))
                        })
                    ]
                }
            }
        );
    }

    #[test]
    fn parse_name_case() {
        assert_eq!(
            syn::parse_str::<NameCase>("Varying").unwrap(),
            NameCase::Varying
        );
        assert_eq!(
            syn::parse_str::<NameCase>("Pascal").unwrap(),
            NameCase::Pascal
        );
        assert_eq!(
            syn::parse_str::<NameCase>("Snake").unwrap(),
            NameCase::Snake
        );
        assert_eq!(
            syn::parse_str::<NameCase>("ScreamingSnake").unwrap(),
            NameCase::ScreamingSnake
        );
        assert_eq!(
            syn::parse_str::<NameCase>("Camel").unwrap(),
            NameCase::Camel
        );
        assert_eq!(
            syn::parse_str::<NameCase>("Kebab").unwrap(),
            NameCase::Kebab
        );
        assert_eq!(
            syn::parse_str::<NameCase>("Cobol").unwrap(),
            NameCase::Cobol
        );
        assert_eq!(
            syn::parse_str::<NameCase>("bla").unwrap_err().to_string(),
            "expected one of: `Varying`, `Pascal`, `Snake`, `ScreamingSnake`, `Camel`, `Kebab`, `Cobol`"
        );
    }

    #[test]
    fn parse_global_config_list() {
        assert_eq!(
            syn::parse_str::<GlobalConfigList>("").unwrap(),
            GlobalConfigList { configs: vec![] }
        );

        assert_eq!(
            syn::parse_str::<GlobalConfigList>("config { }").unwrap(),
            GlobalConfigList { configs: vec![] }
        );

        assert_eq!(
            syn::parse_str::<GlobalConfigList>("config { type DefaultRegisterAccess = RW }")
                .unwrap_err()
                .to_string(),
            "expected `;`"
        );

        assert_eq!(
            syn::parse_str::<GlobalConfigList>("config { type DefaultRegisterAccess = RW; }")
                .unwrap(),
            GlobalConfigList {
                configs: vec![GlobalConfig::DefaultRegisterAccess(Access::RW)]
            }
        );

        assert_eq!(
            syn::parse_str::<GlobalConfigList>(
                "config { type DefaultBufferAccess = RO; type DefaultFieldAccess = RW; }"
            )
            .unwrap(),
            GlobalConfigList {
                configs: vec![
                    GlobalConfig::DefaultBufferAccess(Access::RO),
                    GlobalConfig::DefaultFieldAccess(Access::RW)
                ]
            }
        );

        assert_eq!(
            syn::parse_str::<GlobalConfigList>(
                "config { type DefaultByteOrder = LE; type DefaultBitOrder = LSB0; type NameCase = Camel; }"
            )
            .unwrap(),
            GlobalConfigList {
                configs: vec![
                    GlobalConfig::DefaultByteOrder(ByteOrder::LE),
                    GlobalConfig::DefaultBitOrder(BitOrder::LSB0),
                    GlobalConfig::NameCase(NameCase::Camel)
                ]
            }
        );

        assert_eq!(
            syn::parse_str::<GlobalConfigList>("config { type DefaultRegisterAccesssss = RW; }")
                .unwrap_err()
                .to_string(),
            "expected one of: `DefaultRegisterAccess`, `DefaultFieldAccess`, `DefaultBufferAccess`, `DefaultByteOrder`, `DefaultBitOrder`, `NameCase`"
        );
    }

    #[test]
    fn parse_object() {
        assert_eq!(
            syn::parse_str::<Object>("config { }")
                .unwrap_err()
                .to_string(),
            "expected one of: `block`, `register`, `command`, `buffer`, `ref`"
        );

        assert_eq!(
            syn::parse_str::<Object>("block Foo {}").unwrap(),
            Object::Block(Block {
                attribute_list: AttributeList::new(),
                identifier: Ident::new("Foo", Span::call_site()),
                block_item_list: BlockItemList {
                    block_items: vec![]
                },
                object_list: ObjectList { objects: vec![] }
            }),
        );

        assert_eq!(
            syn::parse_str::<Object>("register Foo {}").unwrap(),
            Object::Register(Register {
                attribute_list: AttributeList::new(),
                identifier: Ident::new("Foo", Span::call_site()),
                register_item_list: RegisterItemList {
                    register_items: vec![]
                },
                field_list: FieldList { fields: vec![] }
            }),
        );

        assert_eq!(
            syn::parse_str::<Object>("command Foo").unwrap(),
            Object::Command(Command {
                attribute_list: AttributeList::new(),
                identifier: Ident::new("Foo", Span::call_site()),
                value: None,
            }),
        );

        assert_eq!(
            syn::parse_str::<Object>("buffer Foo").unwrap(),
            Object::Buffer(Buffer {
                attribute_list: AttributeList::new(),
                identifier: Ident::new("Foo", Span::call_site()),
                access: None,
                address: None,
            }),
        );

        assert_eq!(
            syn::parse_str::<Object>("ref Foo2 = buffer Foo").unwrap(),
            Object::Ref(RefObject {
                identifier: Ident::new("Foo2", Span::call_site()),
                object: Box::new(Object::Buffer(Buffer {
                    attribute_list: AttributeList::new(),
                    identifier: Ident::new("Foo", Span::call_site()),
                    access: None,
                    address: None,
                }))
            }),
        );
    }

    #[test]
    fn parse_device() {
        assert_eq!(
            syn::parse_str::<Device>("").unwrap(),
            Device {
                global_config_list: GlobalConfigList { configs: vec![] },
                object_list: ObjectList { objects: vec![] }
            }
        );

        assert_eq!(
            syn::parse_str::<Device>("config { type DefaultRegisterAccess = RW; }").unwrap(),
            Device {
                global_config_list: GlobalConfigList {
                    configs: vec![GlobalConfig::DefaultRegisterAccess(Access::RW)]
                },
                object_list: ObjectList { objects: vec![] }
            }
        );

        assert_eq!(
            syn::parse_str::<Device>("buffer Foo").unwrap(),
            Device {
                global_config_list: GlobalConfigList { configs: vec![] },
                object_list: ObjectList {
                    objects: vec![Object::Buffer(Buffer {
                        attribute_list: AttributeList::new(),
                        identifier: Ident::new("Foo", Span::call_site()),
                        access: None,
                        address: None,
                    })]
                }
            }
        );

        assert_eq!(
            syn::parse_str::<Device>("config { type DefaultRegisterAccess = RW; }\nbuffer Foo")
                .unwrap(),
            Device {
                global_config_list: GlobalConfigList {
                    configs: vec![GlobalConfig::DefaultRegisterAccess(Access::RW)]
                },
                object_list: ObjectList {
                    objects: vec![Object::Buffer(Buffer {
                        attribute_list: AttributeList::new(),
                        identifier: Ident::new("Foo", Span::call_site()),
                        access: None,
                        address: None,
                    })]
                }
            }
        );
    }
}
