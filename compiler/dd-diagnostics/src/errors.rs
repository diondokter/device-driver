#![allow(
    unused_assignments,
    reason = "Something going on with the diagnostics derive"
)]

use std::borrow::Cow;

use annotate_snippets::{AnnotationKind, Group, Level, Patch, Snippet};
use device_driver_common::{
    identifier::{self, Identifier},
    span::{Span, Spanned},
    specifiers::{BaseType, Integer},
};

use crate::{Diagnostic, encode_ansi_url};

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
        const INFO_TEXT: &str = "no two objects can have the same name. This is true for fields within a field set and variants within an enum";

        [
            Level::ERROR.primary_title("duplicate name found").element(
                Snippet::source(source)
                    .path(path)
                    .annotation(
                        AnnotationKind::Context
                            .span(self.original.into())
                            .label(format!(
                                "the original: {:?}, after word split: {:?}",
                                self.original_value.original(),
                                self.original_value.words().join("·")
                            )),
                    )
                    .annotation(AnnotationKind::Primary.span(self.duplicate.into()).label(
                        format!(
                            "the duplicate: {:?}, after word split: {:?}",
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

pub struct DuplicateVariantValue {
    pub duplicates: Vec<Span>,
    pub value: i128,
}

impl Diagnostic for DuplicateVariantValue {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "all enum variants must have a unique value";

        [
            Level::ERROR
                .primary_title("two or more enum variants share the same value")
                .element(Snippet::source(source).path(path).annotations(
                    self.duplicates.iter().map(|dup| {
                        AnnotationKind::Primary.span(dup.into()).label(format!(
                            "variant value is: {} ({:#X})",
                            self.value, self.value
                        ))
                    }),
                )),
            Group::with_title(Level::INFO.secondary_title(INFO_TEXT)),
        ]
        .to_vec()
    }
}

pub struct EnumBadBasetype {
    pub enum_name: Span,
    pub base_type: Span,
    pub info: &'static str,
    pub context: Vec<Spanned<String>>,
}

impl Diagnostic for EnumBadBasetype {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            Level::ERROR
                .primary_title("invalid base type for enum")
                .element(
                    Snippet::source(source)
                        .path(path)
                        .annotation(
                            AnnotationKind::Primary
                                .span(self.base_type.into())
                                .label("invalid base type"),
                        )
                        .annotation(
                            AnnotationKind::Context
                                .span(self.enum_name.into())
                                .label("enum using invalid base type"),
                        )
                        .annotations(
                            self.context.iter().map(|c| {
                                AnnotationKind::Context.span(c.span.into()).label(&c.value)
                            }),
                        ),
                ),
            Group::with_title(Level::INFO.secondary_title(self.info)),
        ]
        .to_vec()
    }
}

pub struct EnumSizeBitsBiggerThanBaseType {
    pub enum_name: Span,
    pub base_type: Span,
    pub enum_size_bits: u32,
    pub base_type_size_bits: u32,
}

impl Diagnostic for EnumSizeBitsBiggerThanBaseType {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            Level::ERROR
                .primary_title("enum doesn't fit its base type")
                .element(
                    Snippet::source(source)
                        .path(path)
                        .annotation(
                            AnnotationKind::Primary
                                .span(self.enum_name.into())
                                .label(format!("enum is {} bits", self.enum_size_bits)),
                        )
                        .annotation(
                            AnnotationKind::Primary
                                .span(self.base_type.into())
                                .label(format!("base type is {} bits", self.base_type_size_bits)),
                        ),
                ),
            Group::with_title(
                // TODO: Add patch for base type
                Level::HELP.secondary_title("make the enum smaller or pick a bigger base type"),
            ),
        ]
        .to_vec()
    }
}

pub struct EnumNoAutoBaseTypeSelected {
    pub enum_name: Span,
}

impl Diagnostic for EnumNoAutoBaseTypeSelected {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const NOTE_TEXT: &str =
            "a variant or the size-bits is too big to fit in any of the base types";

        [
            Level::ERROR
                .primary_title("no valid base type found")
                .element(
                    Snippet::source(source).path(path).annotation(
                        AnnotationKind::Primary
                            .span(self.enum_name.into())
                            .label("could not select a valid base type for this enum"),
                    ),
                ),
            Group::with_title(Level::NOTE.secondary_title(NOTE_TEXT)),
        ]
        .to_vec()
    }
}

pub struct VariantValuesTooHigh {
    pub variant_names: Vec<Span>,
    pub enum_name: Span,
    pub max_value: i128,
    pub size_bits: u32,
}

impl Diagnostic for VariantValuesTooHigh {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            Level::ERROR
                .primary_title("enum variant value is too high")
                .element(
                    Snippet::source(source)
                        .path(path)
                        .annotation(
                            AnnotationKind::Context
                                .span(self.enum_name.into())
                                .label(format!("enum is {} bits", self.size_bits)),
                        )
                        .annotations(self.variant_names.iter().map(|name| {
                            AnnotationKind::Primary.span(name.into()).label(format!(
                                "variant value exceeds the max of {} ({:#X})",
                                self.max_value, self.max_value
                            ))
                        })),
                ),
            Group::with_title(Level::INFO.secondary_title("all variants must fit in their enum")),
        ]
        .to_vec()
    }
}

pub struct VariantValuesTooLow {
    pub variant_names: Vec<Span>,
    pub enum_name: Span,
    pub min_value: i128,
    pub size_bits: u32,
}

impl Diagnostic for VariantValuesTooLow {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            Level::ERROR
                .primary_title("enum variant value is too low")
                .element(
                    Snippet::source(source)
                        .path(path)
                        .annotation(
                            AnnotationKind::Context
                                .span(self.enum_name.into())
                                .label(format!("enum is {} bits", self.size_bits)),
                        )
                        .annotations(self.variant_names.iter().map(|name| {
                            AnnotationKind::Primary.span(name.into()).label(format!(
                                "variant value exceeds the min of {}",
                                self.min_value
                            ))
                        })),
                ),
            Group::with_title(Level::INFO.secondary_title("all variants must fit in their enum")),
        ]
        .to_vec()
    }
}

pub struct EnumMultipleDefaults {
    pub enum_name: Span,
    pub variant_names: Vec<Span>,
}

impl Diagnostic for EnumMultipleDefaults {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            Level::ERROR
                .primary_title("enum defines more than one default variant")
                .element(
                    Snippet::source(source)
                        .path(path)
                        .annotation(
                            AnnotationKind::Context
                                .span(self.enum_name.into())
                                .label("offending enum"),
                        )
                        .annotations(self.variant_names.iter().enumerate().map(
                            |(index, variant_name)| {
                                if index == 0 {
                                    AnnotationKind::Context
                                        .span(variant_name.into())
                                        .label("first default variant")
                                } else {
                                    AnnotationKind::Primary
                                        .span(variant_name.into())
                                        .label("extra default variant")
                                }
                            },
                        )),
                ),
            Group::with_title(
                Level::INFO.secondary_title("enums can have at most one default variant"),
            ),
        ]
        .to_vec()
    }
}

pub struct EnumMultipleCatchalls {
    pub enum_name: Span,
    pub variant_names: Vec<Span>,
}

impl Diagnostic for EnumMultipleCatchalls {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            Level::ERROR
                .primary_title("enum defines more than one catch-all variant")
                .element(
                    Snippet::source(source)
                        .path(path)
                        .annotation(
                            AnnotationKind::Context
                                .span(self.enum_name.into())
                                .label("offending enum"),
                        )
                        .annotations(self.variant_names.iter().enumerate().map(
                            |(index, variant_name)| {
                                if index == 0 {
                                    AnnotationKind::Context
                                        .span(variant_name.into())
                                        .label("first catch-all variant")
                                } else {
                                    AnnotationKind::Primary
                                        .span(variant_name.into())
                                        .label("extra catch-all variant")
                                }
                            },
                        )),
                ),
            Group::with_title(
                Level::INFO.secondary_title("enums can have at most one catch-all variant"),
            ),
        ]
        .to_vec()
    }
}

pub struct ReferencedObjectDoesNotExist {
    pub object_reference: Span,
}

impl Diagnostic for ReferencedObjectDoesNotExist {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "all objects must be specified in the manifest. It's possible a previous analysis step removed it due to some error. See the previous diagnostics";

        [
            Level::ERROR
                .primary_title("referenced object does not exist")
                .element(
                    Snippet::source(source).path(path).annotation(
                        AnnotationKind::Primary
                            .span(self.object_reference.into())
                            .label("object cannot be found"),
                    ),
                ),
            Group::with_title(Level::INFO.secondary_title(INFO_TEXT)),
        ]
        .to_vec()
    }
}

pub struct InvalidConversionType {
    pub object_reference: Span,
    pub referenced_object: Span,
}

impl Diagnostic for InvalidConversionType {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const NOTE_TEXT: &str = "the referenced object has an invalid type. Only enums and externs can be used for conversions";

        [
            Level::ERROR
                .primary_title("invalid conversion type")
                .element(
                    Snippet::source(source)
                        .path(path)
                        .annotation(
                            AnnotationKind::Primary
                                .span(self.object_reference.into())
                                .label("object referenced as conversion type"),
                        )
                        .annotation(
                            AnnotationKind::Context
                                .span(self.referenced_object.into())
                                .label("referenced object"),
                        ),
                ),
            Group::with_title(Level::NOTE.secondary_title(NOTE_TEXT)),
        ]
        .to_vec()
    }
}

pub struct RepeatEnumWithCatchAll {
    pub repeat_enum: Span,
    pub enum_name: Span,
    pub catch_all: Span,
}

impl Diagnostic for RepeatEnumWithCatchAll {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "to be able to do all analysis passes correctly, the amount of repeats need to be statically known.
This is not possible with an enum containing a catch-all since it can take on any value";

        [
            Level::ERROR
                .primary_title("enum with catch-all used as repeat source")
                .element(
                    Snippet::source(source)
                        .path(path)
                        .annotation(
                            AnnotationKind::Primary
                                .span(self.repeat_enum.into())
                                .label("repeat uses enum with catch-all"),
                        )
                        .annotation(AnnotationKind::Visible.span(self.enum_name.into()))
                        .annotation(
                            AnnotationKind::Context
                                .span(self.catch_all.into())
                                .label("catch-all specified here"),
                        ),
                ),
            Group::with_title(Level::INFO.secondary_title(INFO_TEXT)),
            Group::with_title(Level::HELP.secondary_title(
                "remove the catch-all from the enum or don't use it as repeat source",
            )),
        ]
        .to_vec()
    }
}

pub struct ExternInvalidBaseType {
    pub extern_name: Span,
    pub base_type: Option<Span>,
}

impl Diagnostic for ExternInvalidBaseType {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "externs must specify a fixed size integer type as their base type";

        [
            Level::ERROR
                .primary_title("invalid base type for extern object")
                .element(
                    Snippet::source(source)
                        .path(path)
                        .annotation(
                            if self.base_type.is_some() {
                                AnnotationKind::Context
                            } else {
                                AnnotationKind::Primary
                            }
                            .span(self.extern_name.into())
                            .label(if self.base_type.is_some() {
                                "extern has an invalid base type"
                            } else {
                                "extern has no base type"
                            }),
                        )
                        .annotations(self.base_type.map(|base_type| {
                            AnnotationKind::Primary
                                .span(base_type.into())
                                .label("invalid base type")
                        })),
                ),
            Group::with_title(Level::INFO.secondary_title(INFO_TEXT)),
        ]
        .to_vec()
    }
}

pub struct DifferentBaseTypes {
    pub field: Span,
    pub field_base_type: BaseType,
    pub conversion: Span,
    pub conversion_object: Span,
    pub conversion_base_type: BaseType,
}

impl Diagnostic for DifferentBaseTypes {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "conversions can only happen when the same base type is shared";

        [
            Level::ERROR
                .primary_title("field and conversion use different base types")
                .element(
                    Snippet::source(source)
                        .path(path)
                        .annotation(
                            AnnotationKind::Context
                                .span(self.conversion.into())
                                .label("conversion specified here"),
                        )
                        .annotation(
                            AnnotationKind::Primary
                                .span(self.field.into())
                                .label(format!("field uses base type: {}", self.field_base_type)),
                        )
                        .annotation(
                            AnnotationKind::Primary
                                .span(self.conversion_object.into())
                                .label(format!(
                                    "conversion object uses base type: {}",
                                    self.conversion_base_type
                                )),
                        ),
                ),
            // TODO: Add help with patch
            Group::with_title(Level::INFO.secondary_title(INFO_TEXT)),
        ]
        .to_vec()
    }
}

// TODO: Split in multiple error types
pub struct InvalidInfallibleConversion {
    pub conversion: Span,
    pub context: Vec<Spanned<Cow<'static, str>>>,
    pub existing_type_specifier_content: String,
}

impl Diagnostic for InvalidInfallibleConversion {
    fn is_error(&self) -> bool {
        true
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            Level::ERROR
                .primary_title("invalid infallible conversion")
                .element(
                    Snippet::source(source)
                        .path(path)
                        .annotation(
                            AnnotationKind::Primary
                                .span(self.conversion.into())
                                .label("conversion specified here"),
                        )
                        .annotations(
                            self.context.iter().map(|c| {
                                AnnotationKind::Context.span(c.span.into()).label(&c.value)
                            }),
                        ),
                ),
            // TODO: Add patch
            Group::with_title(Level::HELP.secondary_title("mark the conversion fallible")),
            Group::with_title(
                Level::HELP
                    .secondary_title("make the conversion type support infallible conversion"),
            ),
        ]
        .to_vec()
    }
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
