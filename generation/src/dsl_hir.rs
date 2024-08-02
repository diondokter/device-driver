use syn::{
    braced,
    parse::{Parse, ParseStream},
    Token,
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
        if !input.peek(kw::GlobalConfig) {
            return Ok(Self {
                configs: Vec::new(),
            });
        }

        input.parse::<kw::GlobalConfig>()?;
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
    DefaultRegisterRWType(RWType),
    DefaultFieldRWType(RWType),
    DefaultBufferRWType(RWType),
    DefaultByteOrder(ByteOrder),
    DefaultBitOrder(BitOrder),
    NameCasing(NameCasing),
}

impl Parse for GlobalConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![type]>()?;

        let lookahead = input.lookahead1();

        if lookahead.peek(kw::DefaultRegisterRWType) {
            input.parse::<kw::DefaultRegisterRWType>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            input.parse::<Token![;]>()?;
            Ok(Self::DefaultRegisterRWType(value))
        } else if lookahead.peek(kw::DefaultFieldRWType) {
            input.parse::<kw::DefaultFieldRWType>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            input.parse::<Token![;]>()?;
            Ok(Self::DefaultFieldRWType(value))
        } else if lookahead.peek(kw::DefaultBufferRWType) {
            input.parse::<kw::DefaultBufferRWType>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            input.parse::<Token![;]>()?;
            Ok(Self::DefaultBufferRWType(value))
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

pub enum Object {
    Block(Block),
    Register(Register),
    Command(Command),
    Buffer(Buffer),
    Ref(RefObject),
}

pub struct RefObject {
    pub identifier: syn::Ident,
    pub object: Box<Object>,
}

pub struct AttributeList {
    pub attributes: Vec<Attribute>,
}

pub enum Attribute {
    Doc(syn::LitStr),
    Cfg(proc_macro2::TokenStream),
}

pub struct Block {
    pub attribute_list: AttributeList,
    pub identifier: syn::Ident,
    pub block_item_list: BlockItemList,
    pub object_list: ObjectList,
}

pub struct BlockItemList {
    pub block_items: Vec<BlockItem>,
}

pub enum BlockItem {
    AddressOffset(syn::LitInt),
    Repeat(Repeat),
}

pub struct Register {
    pub attribute_list: AttributeList,
    pub identifier: syn::Ident,
    pub register_item_list: RegisterItemList,
}

pub struct RegisterItemList {
    pub register_items: Vec<RegisterItem>,
}

pub enum RegisterItem {
    RWType(RWType),
    ByteOrder(ByteOrder),
    BitOrder(BitOrder),
    Adress(syn::LitInt),
    SizeBits(syn::LitInt),
    ResetValueInt(syn::LitInt),
    ResetValueArray(Vec<u8>),
    Repeat(Repeat),
    DataField(DataField),
}

pub enum RWType {
    RW,
    RO,
    WO,
}

pub enum ByteOrder {
    LE,
    BE,
}

pub enum BitOrder {
    LSB0,
    MSB0,
}

pub struct DataField {
    pub attribute_list: AttributeList,
    pub identifier: syn::Ident,
    pub base_type: BaseType,
    pub data_field_conversion: Option<DataFieldConversion>,
    pub rw_type: Option<RWType>,
    pub data_field_address: DataFieldAddress,
}

pub enum DataFieldConversion {
    Direct(syn::Ident),
    Enum {
        identifier: syn::Ident,
        enum_variant_list: EnumVariantList,
    },
}

pub struct EnumVariantList {
    pub variants: Vec<EnumVariant>,
}

pub struct EnumVariant {
    pub attribute_list: AttributeList,
    pub identifier: syn::Ident,
    pub enum_value: Option<EnumValue>,
}

pub enum EnumValue {
    Integer(syn::LitInt),
    Default,
    CatchAll,
}

pub enum DataFieldAddress {
    Integer(syn::LitInt),
    Range {
        start: syn::LitInt,
        end: syn::LitInt,
    },
    RangeInclusive {
        start: syn::LitInt,
        end: syn::LitInt,
    },
}

pub enum BaseType {
    Bool,
    Uint,
    Int,
}

pub struct Command {
    pub attribute_list: AttributeList,
    pub identifier: syn::Ident,
    pub value: CommandValue,
}

pub enum CommandValue {
    Basic(syn::LitInt),
    Extended(CommandItemList),
}

pub struct CommandItemList {
    pub items: Vec<CommandItem>,
}

pub enum CommandItem {
    ByteOrder(ByteOrder),
    BitOrder(BitOrder),
    Adress(syn::LitInt),
    SizeBits(syn::LitInt),
    Repeat(Repeat),
    DataFieldIn(DataField),
    DataFieldOut(DataField),
}

pub struct Repeat {
    pub count: syn::LitInt,
    pub stride: syn::LitInt,
}

pub struct Buffer {
    pub attribute_list: AttributeList,
    pub rw_type: Option<RWType>,
    pub address: syn::LitInt,
}

mod kw {
    syn::custom_keyword!(GlobalConfig);
    syn::custom_keyword!(block);
    syn::custom_keyword!(register);
    syn::custom_keyword!(command);
    syn::custom_keyword!(buffer);
    syn::custom_keyword!(count);
    syn::custom_keyword!(stride);
    syn::custom_keyword!(RWType);
    syn::custom_keyword!(ByteOrder);
    syn::custom_keyword!(ADDRESS);
    syn::custom_keyword!(ADDRESS_OFFSET);
    syn::custom_keyword!(SIZE_BITS);
    syn::custom_keyword!(RESET_VALUE);
    syn::custom_keyword!(REPEAT);

    // Global config items
    syn::custom_keyword!(DefaultRegisterRWType);
    syn::custom_keyword!(DefaultFieldRWType);
    syn::custom_keyword!(DefaultBufferRWType);
    syn::custom_keyword!(DefaultByteOrder);
    syn::custom_keyword!(DefaultBitOrder);
    syn::custom_keyword!(NameCasing);

    // NameCasing options
    syn::custom_keyword!(Varying);
    syn::custom_keyword!(PascalCase);
    syn::custom_keyword!(SnakeCase);
    syn::custom_keyword!(ScreamingSnakeCase);
    syn::custom_keyword!(CamelCase);
}
