#![allow(
    unused_assignments,
    reason = "Something going on with the diagnostics derive"
)]

use convert_case::{Case, Casing};
use itertools::Itertools;
use miette::{Diagnostic, LabeledSpan, SourceSpan};
use thiserror::Error;

use crate::mir::{BaseType, FieldConversion, Integer};

#[derive(Error, Debug, Diagnostic)]
#[error("Missing object name")]
#[diagnostic(
    help(
        "The name is specified by the first entry of the node. It must be an anonymous string, e.g. `{object_type} Foo {{ }}`"
    ),
    severity(Error)
)]
pub struct MissingObjectName {
    #[label("For this object")]
    pub object_keyword: SourceSpan,
    #[label("Found this entry instead which is not a valid name")]
    pub found_instead: Option<SourceSpan>,
    pub object_type: String,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Unexpected entries")]
#[diagnostic(
    help(
        "Some entries require a name, require to be anonymous or are expected to be of a certain type. Check the book to see the specification
These entries may also just be superfluous. Try removing them or check other errors to see what's expected"
    ),
    severity(Error)
)]
pub struct UnexpectedEntries {
    #[label(collection, "This entry is unexpected")]
    pub superfluous_entries: Vec<SourceSpan>,
    #[label(collection, "This entry has a name that's unexpected")]
    pub unexpected_name_entries: Vec<SourceSpan>,
    #[label(collection, "This entry was expected to be anonymous")]
    pub not_anonymous_entries: Vec<SourceSpan>,
    #[label(
        collection,
        "This entry was expected to have a name (and not be anonymous)"
    )]
    pub unexpected_anonymous_entries: Vec<SourceSpan>,
}

impl UnexpectedEntries {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.superfluous_entries.is_empty()
            && self.unexpected_name_entries.is_empty()
            && self.not_anonymous_entries.is_empty()
            && self.unexpected_anonymous_entries.is_empty()
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("Unexpected node")]
#[diagnostic(
    help("Expected a node with one of these names: {}", self.print_expected_names()),
    severity(Error)
)]
pub struct UnexpectedNode {
    #[label("The unexpected node")]
    pub node_name: SourceSpan,

    pub expected_names: Vec<&'static str>,
}

impl UnexpectedNode {
    fn print_expected_names(&self) -> String {
        self.expected_names
            .iter()
            .map(|name| format!("`{name}`"))
            .join(", ")
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("Unexpected type")]
#[diagnostic(severity(Error))]
pub struct UnexpectedType {
    #[label("Unexpected type: expected a {}", self.expected_type)]
    pub value_name: SourceSpan,

    pub expected_type: &'static str,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Unexpected value")]
#[diagnostic(severity(Error))]
pub struct UnexpectedValue {
    #[label("Expected one of these values: {}", self.print_expected_values())]
    pub value_name: SourceSpan,

    pub expected_values: Vec<&'static str>,
}

impl UnexpectedValue {
    fn print_expected_values(&self) -> String {
        self.expected_values
            .iter()
            .map(|name| format!("`{name}`"))
            .join(", ")
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("Bad format")]
#[diagnostic(
    help("An example: `{}`", self.example),
    severity(Error)
)]
pub struct BadValueFormat {
    #[label("Value could not be parsed correctly. Use the following format: `{}`", self.expected_format)]
    pub span: SourceSpan,

    pub expected_format: &'static str,
    pub example: &'static str,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Value out of range")]
#[diagnostic(severity(Error))]
pub struct ValueOutOfRange {
    #[label("Value is out of the allowable range: {}", self.range)]
    pub value: SourceSpan,
    #[help]
    pub context: Option<&'static str>,
    pub range: &'static str,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Missing entry")]
#[diagnostic(help("Check the book for all the requirements"), severity(Error))]
pub struct MissingEntry {
    #[label("This node should have one or more of these entries: {}", self.print_expected_entries())]
    pub node_name: SourceSpan,

    pub expected_entries: Vec<&'static str>,
}
impl MissingEntry {
    fn print_expected_entries(&self) -> String {
        self.expected_entries
            .iter()
            .map(|name| format!("`{name}`"))
            .join(", ")
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("Duplicate node")]
#[diagnostic(
    help("This type of node can only appear once. Try removing one of the nodes"),
    severity(Error)
)]
pub struct DuplicateNode {
    #[label(primary, "The duplicate node")]
    pub duplicate: SourceSpan,
    #[label("The original node")]
    pub original: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Duplicate entry")]
#[diagnostic(
    help("This type of entry can only appear once. Try removing one of the entries"),
    severity(Error)
)]
pub struct DuplicateEntry {
    #[label(primary, "The duplicate entry")]
    pub duplicate: SourceSpan,
    #[label("The original entry")]
    pub original: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Empty node")]
#[diagnostic(
    help("This type of node should have children to specify what it contains"),
    severity(Warning)
)]
pub struct EmptyNode {
    #[label("The empty node")]
    pub node: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Missing child node")]
#[diagnostic(help("Check the book to see all required nodes"), severity(Error))]
pub struct MissingChildNode {
    #[label("This {} is missing the required `{}` node", self.node_type.unwrap_or("node"), self.missing_node_type)]
    pub node: SourceSpan,

    pub node_type: Option<&'static str>,
    pub missing_node_type: &'static str,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Inline enum definition without name")]
#[diagnostic(
    help(
        "An inline enum definition is only possible when a conversion is specified, for example: `(uint:EnumName)`"
    ),
    severity(Error)
)]
pub struct InlineEnumDefinitionWithoutName {
    #[label("Add conversion type specification to this field")]
    pub field_name: SourceSpan,
    #[label("A type specifier already exists, but misses the conversion")]
    pub existing_ty: Option<SourceSpan>,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Only base type allowed")]
#[diagnostic(
    help(
        "This object doesn't support the conversion syntax. Just specify the base type and remove the `:{}`",
        self.conversion_text()
    ),
    severity(Error)
)]
pub struct OnlyBaseTypeAllowed {
    #[label("Type specifier contains conversion")]
    pub existing_ty: SourceSpan,
    pub field_conversion: FieldConversion,
}

impl OnlyBaseTypeAllowed {
    fn conversion_text(&self) -> String {
        format!(
            "{}{}",
            self.field_conversion.type_name,
            if self.field_conversion.use_try {
                "?"
            } else {
                ""
            }
        )
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("Address specified in wrong order")]
#[diagnostic(
    help("Addresses are specified with the high bit first and the low bit last"),
    severity(Error)
)]
pub struct AddressWrongOrder {
    #[label("Wrong order, try to change to `@{}:{}`", self.start, self.end)]
    pub address_entry: SourceSpan,
    pub end: u32,
    pub start: u32,
}

#[derive(Error, Debug, Diagnostic)]
#[error("No children were expected for this node")]
#[diagnostic(severity(Error))]
pub struct NoChildrenExpected {
    #[label("Try removing these child nodes")]
    pub children: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Repeat is overspecified")]
#[diagnostic(
    help(
        "A repeat can either use the `count` or the `with` specifier, but not both. Remove one of them."
    ),
    severity(Error)
)]
pub struct RepeatOverSpecified {
    #[label("This repeat has a count")]
    pub count: SourceSpan,
    #[label("This repeat also has a with")]
    pub with: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Field size is too big")]
#[diagnostic(severity(Error), help("A field can be at most 64 bits"))]
pub struct FieldSizeTooBig {
    #[label("{} bits is too big for any of the supported integers", self.size_bits)]
    pub field_address: SourceSpan,
    pub size_bits: u32,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Device name is not Pascal cased")]
#[diagnostic(
    severity(Error),
    help(
        "Device names tend to be a bit weird, so the casing is not automatically changed from the input.\nBut it is required for them to be roughly PascalCase shaped. Try changing it to `{}`", self.suggestion
    )
)]
pub struct DeviceNameNotPascal {
    #[label("This is not Pascal cased. `{}` would be accepted", self.suggestion)]
    pub device_name: SourceSpan,
    pub suggestion: String,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Duplicate name found")]
#[diagnostic(
    severity(Error),
    help(
        "No two objects can have the same name. The same is true for fields in a field set and variants in an enum"
    )
)]
pub struct DuplicateName {
    #[label("The original")]
    pub original: SourceSpan,
    #[label(primary, "The duplicate")]
    pub duplicate: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Enum has no variants")]
#[diagnostic(severity(Error), help("All enums must have at least one variant"))]
pub struct EmptyEnum {
    #[label("Empty enum")]
    pub enum_name: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Two or more enum variants have the same value: {} ({:#X})", self.value, self.value)]
#[diagnostic(severity(Error), help("All enum variants must have a unique value"))]
pub struct DuplicateVariantValue {
    #[label(collection)]
    pub duplicates: Vec<SourceSpan>,
    pub value: i128,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Enum uses an invalid base type")]
#[diagnostic(severity(Error))]
pub struct EnumBadBasetype {
    #[label("Enum with invalid base type")]
    pub enum_name: SourceSpan,
    #[label("Base type being used")]
    pub base_type: SourceSpan,
    #[help]
    pub help: &'static str,
    #[label(collection, "Context")]
    pub context: Vec<LabeledSpan>,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Enum size-bits is bigger than its base type")]
#[diagnostic(
    severity(Error),
    help(
        "The enum is `{size_bits}` bits long, but uses a base type that can't fit that many bits. Use a bigger base type or take a look whether the size-bits is correct"
    )
)]
pub struct EnumSizeBitsBiggerThanBaseType {
    #[label("Enum with too large size-bits or too small base type")]
    pub enum_name: SourceSpan,
    #[label("Base type being used")]
    pub base_type: SourceSpan,
    pub size_bits: u32,
}

#[derive(Error, Debug, Diagnostic)]
#[error("No valid base type could be selected")]
#[diagnostic(
    severity(Error),
    help(
        "Either the specified size-bits or the variants cannot fit within any of the supported integer types"
    )
)]
pub struct EnumNoAutoBaseTypeSelected {
    #[label("Enum with no valid base type")]
    pub enum_name: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("One or more variant values are too high")]
#[diagnostic(
    severity(Error),
    help("The values must fit in the enum integer base type and size-bits. Max = {max_value}")
)]
pub struct VariantValuesTooHigh {
    #[label(collection, "Value too high")]
    pub variant_names: Vec<SourceSpan>,
    #[label("Part of this enum")]
    pub enum_name: SourceSpan,
    pub max_value: i128,
}

#[derive(Error, Debug, Diagnostic)]
#[error("One or more variant values are too low")]
#[diagnostic(
    severity(Error),
    help("The value must fit in the enum integer base type and size-bits. Min = {min_value}")
)]
pub struct VariantValuesTooLow {
    #[label(collection, "Value too low")]
    pub variant_names: Vec<SourceSpan>,
    #[label("Part of this enum")]
    pub enum_name: SourceSpan,
    pub min_value: i128,
}

#[derive(Error, Debug, Diagnostic)]
#[error("More than one default defined on enum")]
#[diagnostic(severity(Error), help("An enum can have at most 1 default variant"))]
pub struct EnumMultipleDefaults {
    #[label("Multiple defaults on this enum")]
    pub enum_name: SourceSpan,
    #[label(collection, "Variant defined as default")]
    pub variant_names: Vec<SourceSpan>,
}

#[derive(Error, Debug, Diagnostic)]
#[error("More than one catch-all defined on enum")]
#[diagnostic(severity(Error), help("An enum can have at most 1 catch-all variant"))]
pub struct EnumMultipleCatchalls {
    #[label("Multiple catch-alls on this enum")]
    pub enum_name: SourceSpan,
    #[label(collection, "Variant defined as catch-all")]
    pub variant_names: Vec<SourceSpan>,
}

#[derive(Error, Debug, Diagnostic)]
#[error("The referenced object does not exist")]
#[diagnostic(
    severity(Error),
    help(
        "All objects must be specified in the manifest. It's possible a previous analysis step removed it due to some error. See the previous diagnostics"
    )
)]
pub struct ReferencedObjectDoesNotExist {
    #[label("This object cannot be found")]
    pub object_reference: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("The referenced object is invalid")]
#[diagnostic(severity(Error))]
pub struct ReferencedObjectInvalid {
    #[label(primary, "Object referenced here has the wrong type")]
    pub object_reference: SourceSpan,
    #[label("The referenced object")]
    pub referenced_object: SourceSpan,
    #[help]
    pub help: String,
}

#[derive(Error, Debug, Diagnostic)]
#[error("The repeat uses an enum that has defined a catch-all")]
#[diagnostic(
    severity(Error),
    help("Repeats have to be statically known. Thus, repeats cannot use enums with a catch-all")
)]
pub struct RepeatEnumWithCatchAll {
    #[label(primary, "Repeat references enum")]
    pub repeat_enum: SourceSpan,
    #[label("Referenced enum")]
    pub enum_name: SourceSpan,
    #[label("The offending catch-all")]
    pub catch_all: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Extern object uses invalid base type")]
#[diagnostic(
    severity(Error),
    help(
        "Externs must use a fixed size integer type as its base type. It cannot be left unspecied either"
    )
)]
pub struct ExternInvalidBaseType {
    #[label(primary, "Extern has invalid base type")]
    pub extern_name: SourceSpan,
    #[label("The invalid base type")]
    pub base_type: Option<SourceSpan>,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Field uses a conversion with a different base type")]
#[diagnostic(
    severity(Error),
    help("A conversion can't change the base type. Make sure they are the same")
)]
pub struct DifferentBaseTypes {
    #[label(primary, "This field uses base type: {field_base_type}")]
    pub field: SourceSpan,
    pub field_base_type: BaseType,
    #[label("It has specified a conversion")]
    pub conversion: SourceSpan,
    #[label("The conversion type uses base type: {conversion_base_type}")]
    pub conversion_object: SourceSpan,
    pub conversion_base_type: BaseType,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Invalid infallible conversion")]
#[diagnostic(
    severity(Error),
    help(
        "Try adding a `?` to mark the conversion as fallible:\n> ({existing_type_specifier_content}?)
        "
    )
)]
pub struct InvalidInfallibleConversion {
    #[label(primary, "Conversion specified here")]
    pub conversion: SourceSpan,
    #[label(collection)]
    pub context: Vec<LabeledSpan>,
    pub existing_type_specifier_content: String,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Unspecied byte order")]
#[diagnostic(
    severity(Error),
    help(
        "Every fieldset larger than 8 bits must specify its byte order. This can be done on each fieldset individually or in the device/global config"
    )
)]
pub struct UnspecifiedByteOrder {
    #[label("No byte order specified. Try adding `byte-order=LE` or `byte-order=BE`")]
    pub fieldset_name: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Reset value is too big")]
#[diagnostic(
    severity(Error),
    help(
        "Reset values must have the same size as their associated register. Integer-specified values are allowed to be smaller and will be 0-padded to the required size"
    )
)]
pub struct ResetValueTooBig {
    #[label(
        "The reset value is specified with {reset_value_size_bits} bits, but the register only has {register_size_bits}"
    )]
    pub reset_value: SourceSpan,
    pub reset_value_size_bits: u32,
    pub register_size_bits: u32,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Reset value is too big")]
#[diagnostic(
    severity(Error),
    help("Reset values must have the same size as their associated register")
)]
pub struct ResetValueArrayWrongSize {
    #[label(
        "The reset value is specified with {reset_value_size_bytes} bytes and the register has {register_size_bytes}"
    )]
    pub reset_value: SourceSpan,
    pub reset_value_size_bytes: u32,
    pub register_size_bytes: u32,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Bool field too large")]
#[diagnostic(
    severity(Error),
    help("A field can only use `bool` as its base type when its size is 1 bit")
)]
pub struct BoolFieldTooLarge {
    #[label("Field set to `bool` here")]
    pub base_type: Option<SourceSpan>,
    #[label("Address is {address_bits} bits")]
    pub address: SourceSpan,
    pub address_bits: u32,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Field has a size of 0")]
#[diagnostic(
    severity(Warning),
    help("The field has no information, so this is likely a mistake")
)]
pub struct ZeroSizeField {
    #[label("Address is {address_bits} bits")]
    pub address: SourceSpan,
    pub address_bits: u32,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Field address exceeds fieldset size")]
#[diagnostic(
    severity(Error),
    help("Fields, including all repeats, must be fully contained in a fieldset")
)]
pub struct FieldAddressExceedsFieldsetSize {
    #[label("Address goes up to {max_field_end}{}", self.get_repeat_message())]
    pub address: SourceSpan,
    pub max_field_end: i128,
    pub repeat_offset: Option<i128>,
    #[label("The fieldset is only {fieldset_size} bits")]
    pub fieldset_size_bits: SourceSpan,
    pub fieldset_size: u32,
}

impl FieldAddressExceedsFieldsetSize {
    fn get_repeat_message(&self) -> String {
        match self.repeat_offset {
            Some(repeat_offset) => format!(" with a repeat offset of {repeat_offset}"),
            None => String::new(),
        }
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("Field address is negative")]
#[diagnostic(
    severity(Error),
    help("Fields, including all repeats, must be fully contained in a fieldset")
)]
pub struct FieldAddressNegative {
    #[label("Address goes down to {min_field_start}{}", self.get_repeat_message())]
    pub address: SourceSpan,
    pub min_field_start: i128,
    pub repeat_offset: Option<i128>,
}

impl FieldAddressNegative {
    fn get_repeat_message(&self) -> String {
        match self.repeat_offset {
            Some(repeat_offset) => format!(" with a repeat offset of {repeat_offset}"),
            None => String::new(),
        }
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("Overlapping fields")]
#[diagnostic(
    severity(Error),
    help(
        "If this was intended, the error can be suppressed by allowing overlap on the fieldset:\n> fieldset Foo allow-bit-overlap {{ }}"
    )
)]
pub struct OverlappingFields {
    #[label("Field sits at address range @{field_address_end_1}:{field_address_start_1}{}", self.get_repeat_message_1())]
    pub field_address_1: SourceSpan,
    pub repeat_offset_1: Option<i128>,
    pub field_address_start_1: i128,
    pub field_address_end_1: i128,
    #[label("Field sits at address range @{field_address_end_2}:{field_address_start_2}{}", self.get_repeat_message_2())]
    pub field_address_2: SourceSpan,
    pub repeat_offset_2: Option<i128>,
    pub field_address_start_2: i128,
    pub field_address_end_2: i128,
}

impl OverlappingFields {
    fn get_repeat_message_1(&self) -> String {
        match self.repeat_offset_1 {
            Some(repeat_offset) => format!(" with a repeat offset of {repeat_offset}"),
            None => String::new(),
        }
    }
    fn get_repeat_message_2(&self) -> String {
        match self.repeat_offset_2 {
            Some(repeat_offset) => format!(" with a repeat offset of {repeat_offset}"),
            None => String::new(),
        }
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("{} address type not defined", object_type.to_case(Case::Pascal))]
#[diagnostic(
    severity(Error),
    help("Add the address type to the config, e.g.:\n> {}-address-type u16", object_type.to_case(Case::Lower))
)]
pub struct AddressTypeUndefined {
    #[label("{} defined here", object_type.to_case(Case::Pascal))]
    pub object: SourceSpan,
    #[label("This device doesn't define a {}-address-type", object_type.to_case(Case::Lower))]
    pub config_device: SourceSpan,
    pub object_type: &'static str,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Address out of range")]
#[diagnostic(
    severity(Error),
    help("Use an address type that fits the whole range being used")
)]
pub struct AddressOutOfRange {
    #[label("Address goes up/down to {address_value}{}", if *has_repeat {" (including repeats)" } else { "" })]
    pub address: SourceSpan,
    pub address_value: i128,
    pub has_repeat: bool,
    #[label("Address type supports a range of {} to {}", address_type.min_value(), address_type.max_value())]
    pub address_type_config: SourceSpan,
    pub address_type: Integer,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Address overlap at {address} ({address:#X})")]
#[diagnostic(
    severity(Error),
    help(
        "If this is intended, the error can be suppressed by allowing overlap on both objects:\n> allow-address-overlap"
    )
)]
pub struct AddressOverlap {
    pub address: i128,
    #[label("Object overlaps with other object{}", if let Some(repeat_offset) = repeat_offset_1 { format!(" at repeat offset {repeat_offset}") } else { String::new() })]
    pub object_1: SourceSpan,
    pub repeat_offset_1: Option<i128>,
    #[label("Object overlaps with other object{}", if let Some(repeat_offset) = repeat_offset_2 { format!(" at repeat offset {repeat_offset}") } else { String::new() })]
    pub object_2: SourceSpan,
    pub repeat_offset_2: Option<i128>,
}
