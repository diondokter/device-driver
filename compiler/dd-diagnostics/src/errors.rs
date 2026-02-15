#![allow(
    unused_assignments,
    reason = "Something going on with the diagnostics derive"
)]

use std::borrow::Cow;

use annotate_snippets::{AnnotationKind, Group, Level, Patch, Snippet};
use device_driver_common::{
    identifier::{self, Identifier},
    span::Span,
    specifiers::{BaseType, Integer, TypeConversion},
};
use itertools::Itertools;
use miette::{Diagnostic as MietteDiagnostic, LabeledSpan};
use thiserror::Error;

use crate::{Diagnostic, encode_ansi_url};

#[derive(Error, Debug, MietteDiagnostic)]
#[error("Missing object name")]
#[diagnostic(
    help(
        "The name is specified by the first entry of the node. It must be an anonymous string, e.g. `{object_type} Foo {{ }}`"
    ),
    severity(Error)
)]
pub struct MissingObjectName {
    #[label("For this object")]
    pub object_keyword: Span,
    #[label("Found this entry instead which is not a valid name")]
    pub found_instead: Option<Span>,
    pub object_type: String,
}

#[derive(Error, Debug, MietteDiagnostic)]
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
    pub superfluous_entries: Vec<Span>,
    #[label(collection, "This entry has a name that's unexpected")]
    pub unexpected_name_entries: Vec<Span>,
    #[label(collection, "This entry was expected to be anonymous")]
    pub not_anonymous_entries: Vec<Span>,
    #[label(
        collection,
        "This entry was expected to have a name (and not be anonymous)"
    )]
    pub unexpected_anonymous_entries: Vec<Span>,
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

#[derive(Error, Debug, MietteDiagnostic)]
#[error("Unexpected node")]
#[diagnostic(
    help("Expected a node with one of these names: {}", self.print_expected_names()),
    severity(Error)
)]
pub struct UnexpectedNode {
    #[label("The unexpected node")]
    pub node_name: Span,

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

#[derive(Error, Debug, MietteDiagnostic)]
#[error("Unexpected type")]
#[diagnostic(severity(Error))]
pub struct UnexpectedType {
    #[label("Unexpected type: expected a {}", self.expected_type)]
    pub value_name: Span,

    pub expected_type: &'static str,
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("Unexpected value")]
#[diagnostic(severity(Error))]
pub struct UnexpectedValue {
    #[label("Expected one of these values: {}", self.print_expected_values())]
    pub value_name: Span,

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

#[derive(Error, Debug, MietteDiagnostic)]
#[error("Bad format")]
#[diagnostic(
    help("An example: `{}`", self.example),
    severity(Error)
)]
pub struct BadValueFormat {
    #[label("Value could not be parsed correctly. Use the following format: `{}`", self.expected_format)]
    pub span: Span,

    pub expected_format: &'static str,
    pub example: &'static str,
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("Value out of range")]
#[diagnostic(severity(Error))]
pub struct ValueOutOfRange {
    #[label("Value is out of the allowable range: {}", self.range)]
    pub value: Span,
    #[help]
    pub context: Option<&'static str>,
    pub range: &'static str,
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("Missing entry")]
#[diagnostic(help("Check the book for all the requirements"), severity(Error))]
pub struct MissingEntry {
    #[label("This node should have one or more of these entries: {}", self.print_expected_entries())]
    pub node_name: Span,

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

#[derive(Error, Debug, MietteDiagnostic)]
#[error("Duplicate node")]
#[diagnostic(
    help("This type of node can only appear once. Try removing one of the nodes"),
    severity(Error)
)]
pub struct DuplicateNode {
    #[label(primary, "The duplicate node")]
    pub duplicate: Span,
    #[label("The original node")]
    pub original: Span,
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("Duplicate entry")]
#[diagnostic(
    help("This type of entry can only appear once. Try removing one of the entries"),
    severity(Error)
)]
pub struct DuplicateEntry {
    #[label(primary, "The duplicate entry")]
    pub duplicate: Span,
    #[label("The original entry")]
    pub original: Span,
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("Empty node")]
#[diagnostic(
    help("This type of node should have children to specify what it contains"),
    severity(Warning)
)]
pub struct EmptyNode {
    #[label("The empty node")]
    pub node: Span,
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("Missing child node")]
#[diagnostic(help("Check the book to see all required nodes"), severity(Error))]
pub struct MissingChildNode {
    #[label("This {} is missing the required `{}` node", self.node_type.unwrap_or("node"), self.missing_node_type)]
    pub node: Span,

    pub node_type: Option<&'static str>,
    pub missing_node_type: &'static str,
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("Inline enum definition without name")]
#[diagnostic(
    help(
        "An inline enum definition is only possible when a conversion is specified, for example: `(uint:EnumName)`"
    ),
    severity(Error)
)]
pub struct InlineEnumDefinitionWithoutName {
    #[label("Add conversion type specification to this field")]
    pub field_name: Span,
    #[label("A type specifier already exists, but misses the conversion")]
    pub existing_ty: Option<Span>,
}

#[derive(Error, Debug, MietteDiagnostic)]
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
    pub existing_ty: Span,
    pub field_conversion: TypeConversion,
}

impl OnlyBaseTypeAllowed {
    fn conversion_text(&self) -> String {
        format!(
            "{}{}",
            self.field_conversion.type_name.original(),
            if self.field_conversion.fallible {
                "?"
            } else {
                ""
            }
        )
    }
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("Address specified in wrong order")]
#[diagnostic(
    help("Addresses are specified with the high bit first and the low bit last"),
    severity(Error)
)]
pub struct AddressWrongOrder {
    #[label("Wrong order, try to change to `@{}:{}`", self.start, self.end)]
    pub address_entry: Span,
    pub end: u32,
    pub start: u32,
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("No children were expected for this node")]
#[diagnostic(severity(Error))]
pub struct NoChildrenExpected {
    #[label("Try removing these child nodes")]
    pub children: Span,
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("Repeat is overspecified")]
#[diagnostic(
    help(
        "A repeat can either use the `count` or the `with` specifier, but not both. Remove one of them."
    ),
    severity(Error)
)]
pub struct RepeatOverSpecified {
    #[label("This repeat has a count")]
    pub count: Span,
    #[label("This repeat also has a with")]
    pub with: Span,
}

pub struct IntegerFieldSizeTooBig {
    pub field_address: Span,
    pub base_type: Span,
    pub field_set: Span,
    pub size_bits: u32,
}

impl Diagnostic for IntegerFieldSizeTooBig {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        let field_message = format!("field has a size of {} bits", self.size_bits);

        [
            Level::ERROR
                .primary_title("field size exceeds 64-bit size limit")
                .element(
                    Snippet::source(source)
                        .path(path)
                        .annotation(
                            AnnotationKind::Primary
                                .span(self.field_address.into())
                                .label(field_message),
                        )
                        .annotation(
                            AnnotationKind::Context
                                .span(self.base_type.into())
                                .label("field uses an integer as base type"),
                        )
                        .annotation(AnnotationKind::Visible.span(self.field_set.into())),
                ),
            Group::with_title(
                Level::NOTE.secondary_title("integer base types are available up to 64-bit"),
            ),
            Group::with_title(Level::INFO.secondary_title(format!(
                "if you need an array or string base type, please comment here: {}",
                encode_ansi_url(
                    "https://github.com/diondokter/device-driver/issues/131",
                    "issue 131"
                )
            ))),
        ]
        .to_vec()
    }
}

pub struct DeviceNameNotPascal {
    pub device_name: Span,
    pub suggestion: String,
}

impl Diagnostic for DeviceNameNotPascal {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "device names tend to be a bit weird, so the casing is not automatically changed from the input. Because of that, they need to be roughly PascalCase shaped.";

        [
            Level::ERROR.primary_title("invalid device name").element(
                Snippet::source(source).path(path).annotation(
                    AnnotationKind::Primary
                        .span(self.device_name.into())
                        .label("device name is not Pascal cased"),
                ),
            ),
            Level::HELP
                .secondary_title("device names need to be pascal-shaped")
                .element(
                    Snippet::source(source)
                        .path(path)
                        .patch(Patch::new(self.device_name.into(), &self.suggestion)),
                ),
            Group::with_title(Level::INFO.secondary_title(INFO_TEXT)),
        ]
        .to_vec()
    }
}

pub struct DuplicateName {
    pub original: Span,
    pub original_value: Identifier,
    pub duplicate: Span,
    pub duplicate_value: Identifier,
}

impl Diagnostic for DuplicateName {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "No two objects can have the same name. This is true for fields within a field set and variants within an enum";

        [
            Level::ERROR.primary_title("Duplicate name found").element(
                Snippet::source(source)
                    .path(path)
                    .annotation(
                        AnnotationKind::Context
                            .span(self.original.into())
                            .label(format!(
                                "The original: {:?}, after word split: {:?}",
                                self.original_value.original(),
                                self.original_value.words().join("·")
                            )),
                    )
                    .annotation(AnnotationKind::Primary.span(self.duplicate.into()).label(
                        format!(
                            "The duplicate: {:?}, after word split: {:?}",
                            self.duplicate_value.original(),
                            self.duplicate_value.words().join("·")
                        ),
                    )),
            ),
            Group::with_title(Level::INFO.secondary_title(INFO_TEXT)),
        ]
        .to_vec()
    }
}

pub struct EmptyEnum {
    pub enum_node: Span,
}

impl Diagnostic for EmptyEnum {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            Level::ERROR.primary_title("enum has no variants").element(
                Snippet::source(source).path(path).annotation(
                    AnnotationKind::Primary
                        .span(self.enum_node.into())
                        .label("empty enum"),
                ),
            ),
            Group::with_title(
                Level::INFO.secondary_title("all enums must have at least one variant"),
            ),
        ]
        .to_vec()
    }
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("Two or more enum variants have the same value: {} ({:#X})", self.value, self.value)]
#[diagnostic(severity(Error), help("All enum variants must have a unique value"))]
pub struct DuplicateVariantValue {
    #[label(collection)]
    pub duplicates: Vec<Span>,
    pub value: i128,
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("Enum uses an invalid base type")]
#[diagnostic(severity(Error))]
pub struct EnumBadBasetype {
    #[label("Enum with invalid base type")]
    pub enum_name: Span,
    #[label("Base type being used")]
    pub base_type: Span,
    #[help]
    pub help: &'static str,
    #[label(collection, "Context")]
    pub context: Vec<LabeledSpan>,
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("Enum size-bits is bigger than its base type")]
#[diagnostic(
    severity(Error),
    help(
        "The enum is `{size_bits}` bits long, but uses a base type that can't fit that many bits. Use a bigger base type or take a look whether the size-bits is correct"
    )
)]
pub struct EnumSizeBitsBiggerThanBaseType {
    #[label("Enum with too large size-bits or too small base type")]
    pub enum_name: Span,
    #[label("Base type being used")]
    pub base_type: Span,
    pub size_bits: u32,
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("No valid base type could be selected")]
#[diagnostic(
    severity(Error),
    help(
        "Either the specified size-bits or the variants cannot fit within any of the supported integer types"
    )
)]
pub struct EnumNoAutoBaseTypeSelected {
    #[label("Enum with no valid base type")]
    pub enum_name: Span,
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("One or more variant values are too high")]
#[diagnostic(
    severity(Error),
    help("The values must fit in the enum integer base type and size-bits. Max = {max_value}")
)]
pub struct VariantValuesTooHigh {
    #[label(collection, "Value too high")]
    pub variant_names: Vec<Span>,
    #[label("Part of this enum")]
    pub enum_name: Span,
    pub max_value: i128,
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("One or more variant values are too low")]
#[diagnostic(
    severity(Error),
    help("The value must fit in the enum integer base type and size-bits. Min = {min_value}")
)]
pub struct VariantValuesTooLow {
    #[label(collection, "Value too low")]
    pub variant_names: Vec<Span>,
    #[label("Part of this enum")]
    pub enum_name: Span,
    pub min_value: i128,
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("More than one default defined on enum")]
#[diagnostic(severity(Error), help("An enum can have at most 1 default variant"))]
pub struct EnumMultipleDefaults {
    #[label("Multiple defaults on this enum")]
    pub enum_name: Span,
    #[label(collection, "Variant defined as default")]
    pub variant_names: Vec<Span>,
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("More than one catch-all defined on enum")]
#[diagnostic(severity(Error), help("An enum can have at most 1 catch-all variant"))]
pub struct EnumMultipleCatchalls {
    #[label("Multiple catch-alls on this enum")]
    pub enum_name: Span,
    #[label(collection, "Variant defined as catch-all")]
    pub variant_names: Vec<Span>,
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("The referenced object does not exist")]
#[diagnostic(
    severity(Error),
    help(
        "All objects must be specified in the manifest. It's possible a previous analysis step removed it due to some error. See the previous diagnostics"
    )
)]
pub struct ReferencedObjectDoesNotExist {
    #[label("This object cannot be found")]
    pub object_reference: Span,
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("The referenced object is invalid")]
#[diagnostic(severity(Error))]
pub struct ReferencedObjectInvalid {
    #[label(primary, "Object referenced here has the wrong type")]
    pub object_reference: Span,
    #[label("The referenced object")]
    pub referenced_object: Span,
    #[help]
    pub help: String,
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("The repeat uses an enum that has defined a catch-all")]
#[diagnostic(
    severity(Error),
    help("Repeats have to be statically known. Thus, repeats cannot use enums with a catch-all")
)]
pub struct RepeatEnumWithCatchAll {
    #[label(primary, "Repeat references enum")]
    pub repeat_enum: Span,
    #[label("Referenced enum")]
    pub enum_name: Span,
    #[label("The offending catch-all")]
    pub catch_all: Span,
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("Extern object uses invalid base type")]
#[diagnostic(
    severity(Error),
    help(
        "Externs must use a fixed size integer type as its base type. It cannot be left unspecied either"
    )
)]
pub struct ExternInvalidBaseType {
    #[label(primary, "Extern has invalid base type")]
    pub extern_name: Span,
    #[label("The invalid base type")]
    pub base_type: Option<Span>,
}

#[derive(Error, Debug, MietteDiagnostic)]
#[error("Field uses a conversion with a different base type")]
#[diagnostic(
    severity(Error),
    help("A conversion can't change the base type. Make sure they are the same")
)]
pub struct DifferentBaseTypes {
    #[label(primary, "This field uses base type: {field_base_type}")]
    pub field: Span,
    pub field_base_type: BaseType,
    #[label("It has specified a conversion")]
    pub conversion: Span,
    #[label("The conversion type uses base type: {conversion_base_type}")]
    pub conversion_object: Span,
    pub conversion_base_type: BaseType,
}

#[derive(Error, Debug, MietteDiagnostic)]
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
    pub conversion: Span,
    #[label(collection)]
    pub context: Vec<LabeledSpan>,
    pub existing_type_specifier_content: String,
}

pub struct UnspecifiedByteOrder {
    pub fieldset_name: Span,
}

impl Diagnostic for UnspecifiedByteOrder {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            Level::ERROR
                .primary_title("unspecified byte order")
                .element(
                    Snippet::source(source).path(path).annotation(
                        AnnotationKind::Primary
                            .span(self.fieldset_name.into())
                            .label("fielset requires a byte order, but none is specified"),
                    ),
                ),
            Group::with_title(Level::HELP.secondary_title(
                "specify the byte order on the fieldset or add a default byte order on the device",
            )), // TODO: Add patch for adding byte order
            Group::with_title(Level::NOTE.secondary_title(
                "the fieldset has a size larger than 8 bits and will span multiple bytes",
            )),
            Group::with_title(Level::INFO.secondary_title(
                "byte order is important for any multi-byte value. It has no default, so it needs to be manually specified",
            )),
        ]
        .to_vec()
    }
}

pub struct ResetValueTooBig {
    pub register_context: Span,
    pub reset_value: Span,
    pub reset_value_size_bits: u32,
    pub register_size_bits: u32,
}

impl Diagnostic for ResetValueTooBig {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "reset values must have the same size as their associated register. Integer-specified values are allowed to be smaller and will be 0-padded to the required size";

        [
            Level::ERROR.primary_title("reset value too big").element(
                Snippet::source(source)
                    .path(path)
                    .annotation(AnnotationKind::Primary.span(self.reset_value.into()).label(
                        format!(
                            "the reset value is specified with {} bits, but the register only has {}",
                            self.reset_value_size_bits, self.register_size_bits
                        ),
                    ))
                    .annotation(AnnotationKind::Visible.span(self.register_context.into())),
            ),
            Group::with_title(Level::INFO.secondary_title(INFO_TEXT)),
        ]
        .to_vec()
    }
}

pub struct ResetValueArrayWrongSize {
    pub register_context: Span,
    pub reset_value: Span,
    pub reset_value_size_bytes: u32,
    pub register_size_bytes: u32,
}

impl Diagnostic for ResetValueArrayWrongSize {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "reset values must have the same size as their associated register";

        [
            Level::ERROR
                .primary_title("reset value wrong size")
                .element(
                Snippet::source(source)
                    .path(path)
                    .annotation(AnnotationKind::Primary.span(self.reset_value.into()).label(
                        format!(
                            "the reset value is specified with {} bytes while the register has {}",
                            self.reset_value_size_bytes, self.register_size_bytes
                        ),
                    ))
                    .annotation(AnnotationKind::Visible.span(self.register_context.into())),
            ),
            Group::with_title(Level::INFO.secondary_title(INFO_TEXT)),
        ]
        .to_vec()
    }
}

pub struct BoolFieldTooLarge {
    pub base_type: Option<Span>,
    pub address: Span,
    pub address_bits: u32,
    pub address_start: u32,
    pub field_set_context: Span,
}

impl Diagnostic for BoolFieldTooLarge {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            Level::ERROR.primary_title("bool field too large").element(
                Snippet::source(source).path(path).annotations(
                    [
                        Some(
                            AnnotationKind::Primary
                                .span(self.address.into())
                                .label(format!("address is {} bits", self.address_bits)),
                        ),
                        self.base_type.map(|base_type| {
                            AnnotationKind::Context
                                .span(base_type.into())
                                .label("bool base type set here")
                        }),
                        Some(AnnotationKind::Visible.span(self.field_set_context.into())),
                    ]
                    .into_iter()
                    .flatten(),
                ),
            ),
            Level::HELP
                .secondary_title("a field with a `bool` base type can only be 1 bit large")
                .element(Snippet::source(source).path(path).patch(Patch::new(
                    self.address.into(),
                    format!("@{}", self.address_start),
                ))),
        ]
        .to_vec()
    }
}

pub struct FieldAddressExceedsFieldsetSize {
    pub address: Span,
    pub max_field_end: i128,
    pub repeat_offset: Option<i128>,
    pub fieldset_size_bits: Span,
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

impl Diagnostic for FieldAddressExceedsFieldsetSize {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            Level::ERROR
                .primary_title("field address exceeds fieldset size")
                .element(
                    Snippet::source(source)
                        .path(path)
                        .annotation(AnnotationKind::Primary.span(self.address.into()).label(
                            format!(
                                "address goes up to {}{}",
                                self.max_field_end,
                                self.get_repeat_message()
                            ),
                        ))
                        .annotation(
                            AnnotationKind::Context
                                .span(self.fieldset_size_bits.into())
                                .label(format!("The fieldset is only {} bits", self.fieldset_size)),
                        ),
                ),
            Group::with_title(Level::INFO.secondary_title(
                "fields, including all repeats, must be fully contained in a fieldset",
            )),
        ]
        .to_vec()
    }
}

pub struct FieldAddressNegative {
    pub address: Span,
    pub min_field_start: i128,
    pub repeat_offset: Option<i128>,
    pub field_set_context: Span,
}

impl FieldAddressNegative {
    fn get_repeat_message(&self) -> Cow<'static, str> {
        match self.repeat_offset {
            Some(repeat_offset) => format!(" with a repeat offset of {repeat_offset}").into(),
            None => "".into(),
        }
    }
}

impl Diagnostic for FieldAddressNegative {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            Level::ERROR
                .primary_title("field address is negative")
                .element(
                    Snippet::source(source)
                        .path(path)
                        .annotation(AnnotationKind::Primary.span(self.address.into()).label(
                            format!(
                                "address goes down to {}{}",
                                self.min_field_start,
                                self.get_repeat_message()
                            ),
                        ))
                        .annotation(AnnotationKind::Visible.span(self.field_set_context.into())),
                ),
            Group::with_title(Level::INFO.secondary_title(
                "fields, including all repeats, must be fully contained in a fieldset",
            )),
        ]
        .to_vec()
    }
}

pub struct OverlappingFields {
    pub field_address_1: Span,
    pub repeat_offset_1: Option<i128>,
    pub field_address_start_1: i128,
    pub field_address_end_1: i128,
    pub field_address_2: Span,
    pub repeat_offset_2: Option<i128>,
    pub field_address_start_2: i128,
    pub field_address_end_2: i128,

    pub field_set_context: Span,
}

impl OverlappingFields {
    fn get_repeat_message_1(&self) -> Cow<'static, str> {
        match self.repeat_offset_1 {
            Some(repeat_offset) => format!(" with a repeat offset of {repeat_offset}").into(),
            None => "".into(),
        }
    }
    fn get_repeat_message_2(&self) -> Cow<'static, str> {
        match self.repeat_offset_2 {
            Some(repeat_offset) => format!(" with a repeat offset of {repeat_offset}").into(),
            None => "".into(),
        }
    }
}

impl Diagnostic for OverlappingFields {
    fn is_error(&self) -> bool {
        false
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const HELP_TEXT: &str = "if overlap is intended, the warning can be suppressed by allowing overlap on both fields";
        const INFO_TEXT: &str = "overlapping fields are usually the result of a copy paste mistake. This warning exists to alert to that possibility";

        [
            Level::WARNING.primary_title("overlapping fields").element(
                Snippet::source(source)
                    .path(path)
                    .annotation(
                        AnnotationKind::Primary
                            .span(self.field_address_1.into())
                            .label(format!(
                                "Field sits at address range @{}:{}{}",
                                self.field_address_end_1 - 1,
                                self.field_address_start_1,
                                self.get_repeat_message_1()
                            )),
                    )
                    .annotation(
                        AnnotationKind::Primary
                            .span(self.field_address_2.into())
                            .label(format!(
                                "Field sits at address range @{}:{}{}",
                                self.field_address_end_2 - 1,
                                self.field_address_start_2,
                                self.get_repeat_message_2()
                            )),
                    )
                    .annotation(AnnotationKind::Visible.span(self.field_set_context.into())),
                // TODO: Add context annotation for where the repeats are defined
            ),
            // TODO: Add patch
            Group::with_title(Level::HELP.secondary_title(HELP_TEXT)),
            Group::with_title(Level::NOTE.secondary_title(INFO_TEXT)),
        ]
        .to_vec()
    }
}

pub struct AddressTypeUndefined {
    pub object_name: Span,
    pub device: Span,
    pub device_config_area: Span,
    pub object_type: &'static str,
}

impl Diagnostic for AddressTypeUndefined {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        vec![
            Level::ERROR
            .primary_title(format!("{} address type not defined", self.object_type))
            .element(
                Snippet::source(source)
                    .path(path)
                    .annotation(
                        AnnotationKind::Primary
                            .span(self.device.into())
                            .label(format!(
                                "this device doesn't define a {}-address-type",
                                self.object_type
                            )),
                    )
                    .annotation(
                        AnnotationKind::Context
                            .span(self.object_name.into())
                            .label(format!("{} object defined here", self.object_type)),
                    ),
            ),
            Level::HELP.secondary_title(
                "add the address type as a global default or as config on the device the object is defined in"
            ).element(
                Snippet::source(source).path(path).patch(
                    Patch::new(self.device_config_area.start..self.device_config_area.start, format!("{}-address-type u16\n", self.object_type))
                )
            ),
            Group::with_title(
                Level::INFO.secondary_title("device-driver is agnostic to the address types being used. As such, it must be manually specified")
            ),
        ]
    }
}

pub struct AddressOutOfRange {
    pub object: Span,
    pub address: Span,
    pub address_value_min: i128,
    pub address_value_max: i128,
    pub address_type_config: Span,
    pub address_type: Integer,
}

impl Diagnostic for AddressOutOfRange {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        vec![
            Level::ERROR
                .primary_title("address out of range")
                .element(
                    Snippet::source(source)
                        .path(path)
                        .annotation(AnnotationKind::Primary.span(self.address.into()).label(
                            if self.address_value_min == self.address_value_max {
                                format!("address has value: {}", self.address_value_max,)
                            } else {
                                format!(
                                    "address ranges from {} to {}",
                                    self.address_value_min, self.address_value_max,
                                )
                            },
                        ))
                        .annotation(AnnotationKind::Visible.span(self.object.into())),
                )
                .element(
                    Snippet::source(source).path(path).annotation(
                        AnnotationKind::Context
                            .span(self.address_type_config.into())
                            .label(format!(
                                "address type supports a range of {} to {}",
                                self.address_type.min_value(),
                                self.address_type.max_value()
                            )),
                    ),
                ),
            if let Some(fitting_integer) =
                Integer::find_smallest(self.address_value_min, self.address_value_max, 0)
            {
                Level::HELP
                    .secondary_title("use an address type that fits the whole range being used")
                    .element(Snippet::source(source).path(path).patch(Patch::new(
                        self.address_type_config.into(),
                        fitting_integer.to_string(),
                    )))
            } else {
                Group::with_title(
                    Level::HELP
                        .secondary_title("address is too big to fit any possible address type"),
                )
            },
        ]
    }
}

pub struct AddressOverlap {
    pub address: i128,
    pub object_1: Span,
    pub object_1_address: Span,
    pub repeat_offset_1: Option<i128>,
    pub object_2: Span,
    pub object_2_address: Span,
    pub repeat_offset_2: Option<i128>,
}

impl Diagnostic for AddressOverlap {
    fn is_error(&self) -> bool {
        false
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        let object_1_message = format!(
            "object overlaps with other object{}",
            if let Some(repeat_offset) = self.repeat_offset_1 {
                format!(" at repeat offset {repeat_offset}")
            } else {
                String::new()
            }
        );
        let object_2_message = format!(
            "object overlaps with other object{}",
            if let Some(repeat_offset) = self.repeat_offset_2 {
                format!(" at repeat offset {repeat_offset}")
            } else {
                String::new()
            }
        );

        const HELP_TEXT: &str = "if overlap is intended, the warning can be suppressed by allowing overlap on both objects";
        const INFO_TEXT: &str = "overlapping objects are usually the result of a copy paste mistake. This warning exists to alert to that possibility";

        [
            Level::WARNING
                .primary_title(format!(
                    "address overlap at {} ({:#X})",
                    self.address, self.address
                ))
                .element(
                    Snippet::source(source)
                        .path(path)
                        .annotation(
                            AnnotationKind::Primary
                                .span(self.object_1.into())
                                .label(object_1_message),
                        )
                        .annotation(
                            AnnotationKind::Context
                                .span(self.object_1_address.into())
                                .label("address set here"),
                        ), // TODO: Add context annotation for where the repeat is defined
                )
                .element(
                    Snippet::source(source)
                        .path(path)
                        .annotation(
                            AnnotationKind::Primary
                                .span(self.object_2.into())
                                .label(object_2_message),
                        )
                        .annotation(
                            AnnotationKind::Context
                                .span(self.object_2_address.into())
                                .label("address set here"),
                        ), // TODO: Add context annotation for where the repeat is defined
                ),
            // TODO: Add patch
            Group::with_title(Level::HELP.secondary_title(HELP_TEXT)),
            Group::with_title(Level::NOTE.secondary_title(INFO_TEXT)),
        ]
        .to_vec()
    }
}

pub struct InvalidIdentifier {
    pub error: identifier::Error,
    pub identifier: Span,
}

impl InvalidIdentifier {
    pub fn new(error: identifier::Error, identifier: Span) -> Self {
        Self { error, identifier }
    }
}

impl Diagnostic for InvalidIdentifier {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "Identifiers are split into words using the name-word-boundaries.\n\
After the split the first character of the first word must be a unicode XID start character.\n\
All other characters must be a unicode XID continue character.";

        let annotation = match self.error {
            identifier::Error::Empty => AnnotationKind::Primary
                .span(self.identifier.into())
                .label("identifier is empty"),
            identifier::Error::EmptyAfterSplits => AnnotationKind::Primary
                .span(self.identifier.into())
                .label("identifier is empty after word split"),
            identifier::Error::InvalidCharacter {
                byte_offset: offset,
                invalid_char: character,
            } if !self.identifier.is_empty() => AnnotationKind::Primary
                .span(
                    self.identifier.start + offset
                        ..self.identifier.start + offset + character.len_utf8(),
                )
                .label(format!(
                    "`{character}` (or `{}`) is not a valid character",
                    character.escape_unicode()
                )),
            identifier::Error::InvalidCharacter {
                byte_offset: _,
                invalid_char: character,
            } => AnnotationKind::Primary
                .span(self.identifier.into())
                .label(format!(
                    "`{character}` (or `{}`) is not a valid character",
                    character.escape_unicode()
                )),
        };

        [
            Level::ERROR
                .primary_title("invalid identifier")
                .element(Snippet::source(source).path(path).annotation(annotation)),
            Group::with_title(Level::INFO.secondary_title(INFO_TEXT)),
        ]
        .to_vec()
    }
}
