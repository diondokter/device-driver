#![allow(
    unused_assignments,
    reason = "Something going on with the diagnostics derive"
)]

use std::borrow::Cow;

use annotate_snippets::{AnnotationKind, Group, Level, Patch, Snippet};
use device_driver_common::{
    identifier::{self, Identifier, RuntimeNamespace},
    interner::Istr,
    span::{Span, Spanned},
    specifiers::{BaseType, Integer, NodeType},
};
use itertools::Itertools;

use crate::{Diagnostic, Severity, encode_ansi_url};

#[derive(Debug)]
pub struct IntegerFieldSizeTooBig {
    pub field_address: Span,
    pub base_type: Span,
    pub field_set: Span,
    pub size_bits: u64,
}

impl Diagnostic for IntegerFieldSizeTooBig {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        let field_message = format!("field has a size of {} bits", self.size_bits);

        [
            self.title_snippet().element(
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

    fn primary_span(&self) -> Span {
        self.field_address
    }

    fn title(&self) -> Cow<'static, str> {
        "field size exceeds 64-bit size limit".into()
    }
}

#[derive(Debug)]
pub struct DeviceNameNotPascal {
    pub device_name: Span,
    pub suggestion: String,
}

impl Diagnostic for DeviceNameNotPascal {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "device names tend to be a bit weird, so the casing is not automatically changed from the input. Because of that, they need to be roughly PascalCase shaped.";

        [
            self.title_snippet().element(
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

    fn primary_span(&self) -> Span {
        self.device_name
    }

    fn title(&self) -> Cow<'static, str> {
        "invalid device name".into()
    }
}

#[derive(Debug)]
pub struct DuplicateName {
    pub original: Span,
    pub original_value: Identifier<RuntimeNamespace>,
    pub duplicate: Span,
    pub duplicate_value: Identifier<RuntimeNamespace>,
}

impl Diagnostic for DuplicateName {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str =
            "names may not collide within their namespace. There are 4 namespaces:
- Types: a type definition
- Operations: something you *do* with a driver
- Fields: unique within a fieldset
- Enum variants: unique within an enum";

        [
            self.title_snippet().element(
                Snippet::source(source)
                    .path(path)
                    .annotation(
                        AnnotationKind::Context
                            .span(self.original.into())
                            .label(format!(
                                "the original: {:?}, after word split: {:?}",
                                self.original_value.original(),
                                self.original_value.words_display()
                            )),
                    )
                    .annotation(AnnotationKind::Primary.span(self.duplicate.into()).label(
                        format!(
                            "the duplicate: {:?}, after word split: {:?}",
                            self.duplicate_value.original(),
                            self.duplicate_value.words_display()
                        ),
                    )),
            ),
            Group::with_title(Level::INFO.secondary_title(INFO_TEXT)),
        ]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.duplicate
    }

    fn title(&self) -> Cow<'static, str> {
        "duplicate name found".into()
    }
}

#[derive(Debug)]
pub struct EmptyEnum {
    pub enum_node: Span,
}

impl Diagnostic for EmptyEnum {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            self.title_snippet().element(
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

    fn primary_span(&self) -> Span {
        self.enum_node
    }

    fn title(&self) -> Cow<'static, str> {
        "enum has no variants".into()
    }
}

#[derive(Debug)]
pub struct DuplicateVariantValue {
    pub duplicates: Vec<Span>,
    pub value: i128,
}

impl Diagnostic for DuplicateVariantValue {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "all enum variants must have a unique value";

        [
            self.title_snippet()
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

    fn primary_span(&self) -> Span {
        self.duplicates[0]
    }

    fn title(&self) -> Cow<'static, str> {
        "two or more enum variants share the same value".into()
    }
}

#[derive(Debug)]
pub struct EnumBadBasetype {
    pub enum_name: Span,
    pub base_type: Span,
    pub info: &'static str,
    pub context: Vec<Spanned<String>>,
}

impl Diagnostic for EnumBadBasetype {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            self.title_snippet().element(
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
                        self.context
                            .iter()
                            .map(|c| AnnotationKind::Context.span(c.span.into()).label(&c.value)),
                    ),
            ),
            Group::with_title(Level::INFO.secondary_title(self.info)),
        ]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.base_type
    }

    fn title(&self) -> Cow<'static, str> {
        "invalid base type for enum".into()
    }
}

#[derive(Debug)]
pub struct EnumSizeBitsBiggerThanBaseType {
    pub enum_name: Span,
    pub base_type: Span,
    pub enum_size_bits: u32,
    pub base_type_size_bits: u32,
}

impl Diagnostic for EnumSizeBitsBiggerThanBaseType {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            self.title_snippet().element(
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

    fn primary_span(&self) -> Span {
        self.enum_name
    }

    fn title(&self) -> Cow<'static, str> {
        "enum doesn't fit its base type".into()
    }
}

#[derive(Debug)]
pub struct EnumNoAutoBaseTypeSelected {
    pub enum_name: Span,
}

impl Diagnostic for EnumNoAutoBaseTypeSelected {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const NOTE_TEXT: &str =
            "a variant or the size-bits is too big to fit in any of the base types";

        [
            self.title_snippet().element(
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

    fn primary_span(&self) -> Span {
        self.enum_name
    }

    fn title(&self) -> Cow<'static, str> {
        "no valid base type found".into()
    }
}

#[derive(Debug)]
pub struct VariantValuesTooHigh {
    pub variant_names: Vec<Span>,
    pub enum_name: Span,
    pub max_value: i128,
    pub size_bits: u32,
}

impl Diagnostic for VariantValuesTooHigh {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            self.title_snippet().element(
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

    fn primary_span(&self) -> Span {
        self.variant_names[0]
    }

    fn title(&self) -> Cow<'static, str> {
        "enum variant value is too high".into()
    }
}

#[derive(Debug)]
pub struct VariantValuesTooLow {
    pub variant_names: Vec<Span>,
    pub enum_name: Span,
    pub min_value: i128,
    pub size_bits: u32,
}

impl Diagnostic for VariantValuesTooLow {
    fn severity(&self) -> Severity {
        Severity::Error
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

    fn primary_span(&self) -> Span {
        self.variant_names[0]
    }

    fn title(&self) -> Cow<'static, str> {
        "enum variant value is too low".into()
    }
}

#[derive(Debug)]
pub struct EnumMultipleDefaults {
    pub enum_name: Span,
    pub variant_names: Vec<Span>,
}

impl Diagnostic for EnumMultipleDefaults {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            self.title_snippet().element(
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

    fn primary_span(&self) -> Span {
        self.variant_names[1]
    }

    fn title(&self) -> Cow<'static, str> {
        "enum defines more than one default variant".into()
    }
}

#[derive(Debug)]
pub struct EnumMultipleCatchalls {
    pub enum_name: Span,
    pub variant_names: Vec<Span>,
}

impl Diagnostic for EnumMultipleCatchalls {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            self.title_snippet().element(
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

    fn primary_span(&self) -> Span {
        self.variant_names[1]
    }

    fn title(&self) -> Cow<'static, str> {
        "enum defines more than one catch-all variant".into()
    }
}

#[derive(Debug)]
pub struct ReferencedObjectDoesNotExist {
    pub object_reference: Span,
}

impl Diagnostic for ReferencedObjectDoesNotExist {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "all objects must be specified in the manifest. It's possible a previous analysis step removed it due to some error. See the previous diagnostics";

        [
            self.title_snippet().element(
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

    fn primary_span(&self) -> Span {
        self.object_reference
    }

    fn title(&self) -> Cow<'static, str> {
        "referenced object does not exist".into()
    }
}

#[derive(Debug)]
pub struct InvalidConversionType {
    pub object_reference: Span,
    pub referenced_object: Span,
}

impl Diagnostic for InvalidConversionType {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const NOTE_TEXT: &str = "the referenced object has an invalid type. Only enums and externs can be used for conversions";

        [
            self.title_snippet().element(
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

    fn primary_span(&self) -> Span {
        self.object_reference
    }

    fn title(&self) -> Cow<'static, str> {
        "object has invalid conversion type".into()
    }
}

#[derive(Debug)]
pub struct RepeatEnumWithCatchAll {
    pub repeat_enum: Span,
    pub enum_name: Span,
    pub catch_all: Span,
}

impl Diagnostic for RepeatEnumWithCatchAll {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "to be able to do all analysis passes correctly, the amount of repeats need to be statically known.
This is not possible with an enum containing a catch-all since it can take on any value";

        [
            self.title_snippet().element(
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

    fn primary_span(&self) -> Span {
        self.repeat_enum
    }

    fn title(&self) -> Cow<'static, str> {
        "enum with catch-all used as repeat source".into()
    }
}

#[derive(Debug)]
pub struct RepeatMathOverflow {
    pub repeat_span: Span,
    pub max_value_span: Span,
    pub max_value: i128,
    pub stride: i128,
}

impl Diagnostic for RepeatMathOverflow {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "repeat math is done with `i32` integers to keep the runtime lean, so all calculations need to fit in a limited range";

        [
            self.title_snippet().element(
                Snippet::source(source)
                    .path(path)
                    .annotation(AnnotationKind::Primary.span(self.repeat_span.into()).label(
                        format!(
                            "repeat calculation overflows the allowed `i32` range at {}",
                            self.max_value * self.stride
                        ),
                    ))
                    .annotation(
                        AnnotationKind::Context
                            .span(self.max_value_span.into())
                            .label(format!("biggest index of {} specified here, which gets multiplied with the stride", self.max_value)),
                    ),
            ),
            Group::with_title(Level::INFO.secondary_title(INFO_TEXT)),
        ]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.repeat_span
    }

    fn title(&self) -> Cow<'static, str> {
        "repeat math overflow".into()
    }
}

#[derive(Debug)]
pub struct ExternInvalidBaseType {
    pub extern_name: Span,
    pub base_type: Option<Span>,
}

impl Diagnostic for ExternInvalidBaseType {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "externs must specify a fixed size integer type as their base type";

        [
            self.title_snippet().element(
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

    fn primary_span(&self) -> Span {
        self.base_type.unwrap_or(self.extern_name)
    }

    fn title(&self) -> Cow<'static, str> {
        "invalid base type for extern object".into()
    }
}

#[derive(Debug)]
pub struct ExternInvalidSizeBits {
    pub extern_name: Span,
    pub size_bits: Span,
    pub reason: Cow<'static, str>,
}

impl Diagnostic for ExternInvalidSizeBits {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [self.title_snippet().element(
            Snippet::source(source)
                .path(path)
                .annotation(
                    AnnotationKind::Primary
                        .span(self.size_bits.into())
                        .label(&self.reason),
                )
                .annotation(AnnotationKind::Visible.span(self.extern_name.into())),
        )]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.size_bits
    }

    fn title(&self) -> Cow<'static, str> {
        "invalid size-bits value for extern object".into()
    }
}

#[derive(Debug)]
pub struct DifferentBaseTypes {
    pub field: Span,
    pub field_base_type: BaseType,
    pub conversion: Span,
    pub conversion_object: Span,
    pub conversion_base_type: BaseType,
}

impl Diagnostic for DifferentBaseTypes {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "conversions can only happen when the same base type is shared";

        [
            self.title_snippet().element(
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

    fn primary_span(&self) -> Span {
        self.field
    }

    fn title(&self) -> Cow<'static, str> {
        "field and conversion use different base types".into()
    }
}

// TODO: Split in multiple error types
#[derive(Debug)]
pub struct InvalidInfallibleConversion {
    pub field: Span,
    pub conversion: Span,
    pub context: Vec<Spanned<Cow<'static, str>>>,
    pub existing_type_specifier_content: String,
}

impl Diagnostic for InvalidInfallibleConversion {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            self.title_snippet().element(
                Snippet::source(source)
                    .path(path)
                    .annotation(
                        AnnotationKind::Primary
                            .span(self.conversion.into())
                            .label("conversion specified here"),
                    )
                    .annotations(
                        self.context
                            .iter()
                            .map(|c| AnnotationKind::Context.span(c.span.into()).label(&c.value)),
                    )
                    .annotation(AnnotationKind::Visible.span(self.field.into())),
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

    fn primary_span(&self) -> Span {
        self.conversion
    }

    fn title(&self) -> Cow<'static, str> {
        "invalid infallible conversion".into()
    }
}

#[derive(Debug)]
pub struct ConversionTypeTooBig {
    pub field: Span,
    pub field_address: Span,
    pub conversion_type: Span,
    pub conversion: Span,
    pub field_len: u64,
    pub conversion_len: u64,
}

impl Diagnostic for ConversionTypeTooBig {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "a field can only convert to types of equal length or smaller";

        [
            self.title_snippet().element(
                Snippet::source(source)
                    .path(path)
                    .annotation(AnnotationKind::Visible.span(self.field.into()))
                    .annotation(
                        AnnotationKind::Primary
                            .span(self.field_address.into())
                            .label(format!("field is {} bits", self.field_len)),
                    )
                    .annotation(
                        AnnotationKind::Primary
                            .span(self.conversion_type.into())
                            .label(format!("target type is {} bits", self.conversion_len)),
                    )
                    .annotation(
                        AnnotationKind::Context
                            .span(self.conversion.into())
                            .label("field specifies a conversion type here"),
                    ),
            ),
            Group::with_title(Level::INFO.secondary_title(INFO_TEXT)),
        ]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.field_address
    }

    fn title(&self) -> Cow<'static, str> {
        "conversion type too big for field".into()
    }
}

#[derive(Debug)]
pub struct UnspecifiedByteOrder {
    pub fieldset_name: Span,
    pub properties_span: Option<Span>,
}

impl Diagnostic for UnspecifiedByteOrder {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            self.title_snippet()
                .element(
                    Snippet::source(source).path(path).annotation(
                        AnnotationKind::Primary
                            .span(self.fieldset_name.into())
                            .label("fieldset requires a byte order, but none is specified"),
                    ),
                ),
            Level::HELP.secondary_title(
                "specify the byte order on the fieldset or add a default byte order on the device",
            )
            .elements( self.properties_span.map(|properties_span| {
                Snippet::source(source).path(path).patch(
                    Patch::new(properties_span.start..properties_span.start, "byte-order: LE,\n")
                )}
            )),
            Group::with_title(Level::NOTE.secondary_title(
                "the fieldset spans multiple bytes, so it needs to have byte ordering specified",
            )),
            Group::with_title(Level::INFO.secondary_title(
                "byte order is important for any multi-byte value. It has no default, so it needs to be manually specified",
            )),
        ]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.fieldset_name
    }

    fn title(&self) -> Cow<'static, str> {
        "fieldset needs byte order specified".into()
    }
}

#[derive(Debug)]
pub struct UnspecifiedAccess {
    pub object_name: Span,
    pub short_property: bool,
    pub properties_span: Option<Span>,
}

impl Diagnostic for UnspecifiedAccess {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            self.title_snippet().element(
                Snippet::source(source).path(path).annotation(
                    AnnotationKind::Primary
                        .span(self.object_name.into())
                        .label("object requires an access to be specified, but none is"),
                ),
            ),
            Level::HELP
                .secondary_title(
                    "specify the access on the object or add a `default-access` to a parent object",
                )
                .elements(self.properties_span.map(|properties_span| {
                    Snippet::source(source).path(path).patch(Patch::new(
                        if self.short_property {
                            properties_span.end..properties_span.end
                        } else {
                            properties_span.start..properties_span.start
                        },
                        if self.short_property {
                            " RW"
                        } else {
                            "access: RW,\n"
                        },
                    ))
                })),
        ]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.object_name
    }

    fn title(&self) -> Cow<'static, str> {
        "object needs access specified".into()
    }
}

#[derive(Debug)]
pub struct ResetValueIntTooBig {
    pub register_context: Span,
    pub reset_value: Span,
    pub reset_value_size_bytes: u32,
    pub register_size_bytes: u32,
}

impl Diagnostic for ResetValueIntTooBig {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "reset values cannot be bigger than their fieldset";

        [
            self.title_snippet().element(
                Snippet::source(source)
                    .path(path)
                    .annotation(AnnotationKind::Primary.span(self.reset_value.into()).label(
                        format!(
                            "the reset value is specified with {} bytes, but the register only has {}",
                            self.reset_value_size_bytes, self.register_size_bytes
                        ),
                    ))
                    .annotation(AnnotationKind::Visible.span(self.register_context.into())),
            ),
            Group::with_title(Level::INFO.secondary_title(INFO_TEXT)),
        ]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.reset_value
    }

    fn title(&self) -> Cow<'static, str> {
        "reset value too big for register".into()
    }
}

#[derive(Debug)]
pub struct ResetValueArrayWrongSize {
    pub register_context: Span,
    pub reset_value: Span,
    pub reset_value_size_bytes: u32,
    pub register_size_bytes: u32,
}

impl Diagnostic for ResetValueArrayWrongSize {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "reset values must have the same size as their associated register";

        [
            self.title_snippet().element(
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

    fn primary_span(&self) -> Span {
        self.reset_value
    }

    fn title(&self) -> Cow<'static, str> {
        "reset value wrong size".into()
    }
}

#[derive(Debug)]
pub struct BoolFieldTooLarge {
    pub base_type: Option<Span>,
    pub address: Span,
    pub address_bits: u32,
    pub address_start: u32,
    pub field_set_context: Span,
}

impl Diagnostic for BoolFieldTooLarge {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            self.title_snippet().element(
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
                    format!("{}:{}", self.address_start, self.address_start),
                )))
                .element(Snippet::source(source).path(path).patch(Patch::new(
                    self.address.into(),
                    format!("{}", self.address_start),
                ))),
        ]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.address
    }

    fn title(&self) -> Cow<'static, str> {
        "bool field too large".into()
    }
}

#[derive(Debug)]
pub struct FieldAddressExceedsFieldsetSize {
    pub address: Span,
    pub max_field_end: i128,
    pub repeat_offset: Option<i128>,
    pub fieldset_size_span: Span,
    pub fieldset_size_bits: u32,
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
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            self.title_snippet().element(
                Snippet::source(source)
                    .path(path)
                    .annotation(
                        AnnotationKind::Primary
                            .span(self.address.into())
                            .label(format!(
                                "address goes up to {}{}",
                                self.max_field_end,
                                self.get_repeat_message()
                            )),
                    )
                    .annotation(
                        AnnotationKind::Context
                            .span(self.fieldset_size_span.into())
                            .label(format!(
                                "The fieldset is only {} bits",
                                self.fieldset_size_bits
                            )),
                    ),
            ),
            Group::with_title(Level::INFO.secondary_title(
                "fields, including all repeats, must be fully contained in a fieldset",
            )),
        ]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.address
    }

    fn title(&self) -> Cow<'static, str> {
        "field address exceeds fieldset size".into()
    }
}

#[derive(Debug)]
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
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            self.title_snippet().element(
                Snippet::source(source)
                    .path(path)
                    .annotation(
                        AnnotationKind::Primary
                            .span(self.address.into())
                            .label(format!(
                                "address goes down to {}{}",
                                self.min_field_start,
                                self.get_repeat_message()
                            )),
                    )
                    .annotation(AnnotationKind::Visible.span(self.field_set_context.into())),
            ),
            Group::with_title(Level::INFO.secondary_title(
                "fields, including all repeats, must be fully contained in a fieldset",
            )),
        ]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.address
    }

    fn title(&self) -> Cow<'static, str> {
        "field address is negative".into()
    }
}

#[derive(Debug)]
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
    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const HELP_TEXT: &str = "if overlap is intended, the warning can be suppressed by allowing overlap on both fields";
        const INFO_TEXT: &str = "overlapping fields are usually the result of a copy paste mistake. This warning exists to alert to that possibility";

        [
            self.title_snippet().element(
                Snippet::source(source)
                    .path(path)
                    .annotation(
                        AnnotationKind::Primary
                            .span(self.field_address_1.into())
                            .label(format!(
                                "field sits at address range @{}:{}{}",
                                self.field_address_end_1 - 1,
                                self.field_address_start_1,
                                self.get_repeat_message_1()
                            )),
                    )
                    .annotation(
                        AnnotationKind::Primary
                            .span(self.field_address_2.into())
                            .label(format!(
                                "field sits at address range @{}:{}{}",
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

    fn primary_span(&self) -> Span {
        self.field_address_2
    }

    fn title(&self) -> Cow<'static, str> {
        "overlapping fields".into()
    }
}

#[derive(Debug)]
pub struct AddressTypeUndefined {
    pub object_name: Span,
    pub device: Span,
    pub properties_span: Option<Span>,
    pub object_type: &'static str,
}

impl Diagnostic for AddressTypeUndefined {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        vec![
            self.title_snippet().element(
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
            ).elements(
                self.properties_span.map(|properties_span| {
                    Snippet::source(source).path(path).patch(
                        Patch::new(properties_span.start..properties_span.start, format!("{}-address-type: u16\n", self.object_type))
                    )
                })
            ),
            Group::with_title(
                Level::INFO.secondary_title("device-driver is agnostic to the address types being used. As such, it must be manually specified")
            ),
        ]
    }

    fn primary_span(&self) -> Span {
        self.device
    }

    fn title(&self) -> Cow<'static, str> {
        format!("{} address type not defined", self.object_type).into()
    }
}

#[derive(Debug)]
pub struct AddressOutOfRange {
    pub object: Span,
    pub address: Span,
    pub address_value_min: i128,
    pub address_value_max: i128,
    pub address_type_config: Span,
    pub address_type: Integer,
}

impl Diagnostic for AddressOutOfRange {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        vec![
            self.title_snippet()
                .element(
                    Snippet::source(source)
                        .path(path)
                        .annotation(AnnotationKind::Primary.span(self.address.into()).label(
                            if self.address_value_min == self.address_value_max {
                                format!("address has value: {}", self.address_value_max)
                            } else {
                                format!(
                                    "address ranges from {} to {}",
                                    self.address_value_min, self.address_value_max
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

    fn primary_span(&self) -> Span {
        self.address
    }

    fn title(&self) -> Cow<'static, str> {
        format!(
            "address out of range for address type {}",
            self.address_type
        )
        .into()
    }
}

#[derive(Debug)]
pub struct AddressOverlap {
    pub address: i128,
    pub object_1: Span,
    pub object_1_address: Span,
    pub object_1_size: Span,
    pub repeat_offset_1: Option<i128>,
    pub object_2: Span,
    pub object_2_address: Span,
    pub object_2_size: Span,
    pub repeat_offset_2: Option<i128>,
}

impl Diagnostic for AddressOverlap {
    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        let object_1_message = format!(
            "object 1 overlaps with other object 2{}",
            if let Some(repeat_offset) = self.repeat_offset_1 {
                format!(" at repeat offset {repeat_offset}")
            } else {
                String::new()
            }
        );
        let object_2_message = format!(
            "object 2 overlaps with other object 1{}",
            if let Some(repeat_offset) = self.repeat_offset_2 {
                format!(" at repeat offset {repeat_offset}")
            } else {
                String::new()
            }
        );

        const HELP_TEXT: &str = "if overlap is intended, the warning can be suppressed by allowing overlap on both objects";
        const INFO_TEXT: &str = "overlapping objects are usually the result of a copy paste mistake. This warning exists to alert to that possibility";

        [
            self.title_snippet()
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
                                .label("address 1 set here"),
                        )
                        .annotations(
                            (!self.object_1_size.is_empty()).then_some(
                                AnnotationKind::Context
                                    .span(self.object_1_size.into())
                                    .label("size 1 set here"),
                            ),
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
                                .label("address 2 set here"),
                        )
                        .annotations(
                            (!self.object_2_size.is_empty()).then_some(
                                AnnotationKind::Context
                                    .span(self.object_2_size.into())
                                    .label("size 2 set here"),
                            ),
                        ), // TODO: Add context annotation for where the repeat is defined
                ),
            // TODO: Add patch
            Group::with_title(Level::HELP.secondary_title(HELP_TEXT)),
            Group::with_title(Level::NOTE.secondary_title(INFO_TEXT)),
        ]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.object_2
    }

    fn title(&self) -> Cow<'static, str> {
        format!("address overlap at {} ({:#X})", self.address, self.address).into()
    }
}

#[derive(Debug)]
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
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "identifiers are split into words using the 'word-boundaries'.\n\
After the split the first character of the first word must be a unicode XID start character.\n\
All other characters must be a unicode XID continue character.\n\
\n\
Identifiers must also be able to be converted to different casings. That means the split words must not contain `-` or `_`,\n\
which in practice means the word-boundaries should always include those characters.";

        let annotation = match &self.error {
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
            e @ identifier::Error::CannotConvert { .. } => AnnotationKind::Primary
                .span(self.identifier.into())
                .label(e.to_string()),
        };

        [
            self.title_snippet()
                .element(Snippet::source(source).path(path).annotation(annotation)),
            Group::with_title(Level::INFO.secondary_title(INFO_TEXT)),
        ]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.identifier
    }

    fn title(&self) -> Cow<'static, str> {
        "invalid identifier".into()
    }
}

#[derive(Debug)]
pub struct InvalidAutoIdentifier {
    pub auto_identifier: Span,
}

impl Diagnostic for InvalidAutoIdentifier {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "auto identifiers can only be used in places where there's a parent node of which the name can be taken";

        [
            self.title_snippet().element(
                Snippet::source(source).path(path).annotation(
                    AnnotationKind::Primary
                        .span(self.auto_identifier.into())
                        .label("auto identifier can't be used here"),
                ),
            ),
            Group::with_title(Level::INFO.secondary_title(INFO_TEXT)),
        ]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.auto_identifier
    }

    fn title(&self) -> Cow<'static, str> {
        "invalid identifier".into()
    }
}

#[derive(Debug)]
pub struct ParsingError {
    pub reason: String,
    pub span: Span,
}

impl Diagnostic for ParsingError {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [Level::ERROR.primary_title("parsing error").element(
            Snippet::source(source).path(path).annotation(
                AnnotationKind::Primary
                    .span(self.span.into())
                    .label(&self.reason),
            ),
        )]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.span
    }

    fn title(&self) -> Cow<'static, str> {
        format!("parsing error: {}", self.reason).into()
    }
}

#[derive(Debug)]
pub struct UnknownNodeType {
    pub node_type: Span,
    pub allowed_node_types: Vec<NodeType>,
}

impl Diagnostic for UnknownNodeType {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [self.title_snippet().element(
            Snippet::source(source).path(path).annotation(
                AnnotationKind::Primary
                    .span(self.node_type.into())
                    .label(format!(
                        "expected one of: {}",
                        self.allowed_node_types.iter().join(", ")
                    )),
            ),
        )]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.node_type
    }

    fn title(&self) -> Cow<'static, str> {
        "unknown node type".into()
    }
}

#[derive(Debug)]
pub struct InvalidPropertyName {
    pub property: Span,
    pub node_type: Spanned<NodeType>,
    pub expected_names: Vec<Istr>,
}

impl Diagnostic for InvalidPropertyName {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [self
            .title_snippet()
            .element(Snippet::source(source).path(path).annotation(
                AnnotationKind::Primary.span(self.property.into()).label(
                    if self.expected_names.is_empty() {
                        "no named properties are expected".into()
                    } else {
                        format!("expected one of: {}", self.expected_names.join(", "))
                    },
                ),
            ))]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.property
    }

    fn title(&self) -> Cow<'static, str> {
        format!("invalid property name for `{}` nodes", self.node_type).into()
    }
}

#[derive(Debug)]
pub struct InvalidExpressionType {
    pub expression: Spanned<String>,
    pub node_type: Spanned<NodeType>,
    pub valid_expression_types: Vec<String>,
    pub valid_expression_values: Vec<Cow<'static, str>>,
}

impl Diagnostic for InvalidExpressionType {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        let mut report = [self.title_snippet().element(
            Snippet::source(source).path(path).annotation(
                AnnotationKind::Primary
                    .span(self.expression.span.into())
                    .label(format!(
                        "got {}, expected one of: {}",
                        self.expression,
                        self.valid_expression_types.join(", ")
                    )),
            ),
        )]
        .to_vec();

        for (name, value) in self
            .valid_expression_types
            .iter()
            .zip(&self.valid_expression_values)
        {
            report.push(
                Level::HELP
                    .secondary_title(format!("change to a {name} expression"))
                    .element(
                        Snippet::source(source)
                            .path(path)
                            .patch(Patch::new(self.expression.span.into(), &**value)),
                    ),
            );
        }

        report
    }

    fn primary_span(&self) -> Span {
        self.expression.span
    }

    fn title(&self) -> Cow<'static, str> {
        format!(
            "invalid expression type for this property in {} nodes",
            self.node_type
        )
        .into()
    }
}

#[derive(Debug)]
pub struct DuplicateProperty {
    pub original: Span,
    pub duplicate: Span,
}

impl Diagnostic for DuplicateProperty {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [self.title_snippet().element(
            Snippet::source(source)
                .path(path)
                .annotation(
                    AnnotationKind::Context
                        .span(self.original.into())
                        .label("first occurrence"),
                )
                .annotation(
                    AnnotationKind::Primary
                        .span(self.duplicate.into())
                        .label("duplicate"),
                ),
        )]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.duplicate
    }

    fn title(&self) -> Cow<'static, str> {
        "duplicate property".into()
    }
}

#[derive(Debug)]
pub struct InvalidNodeType {
    pub node_type: Span,
    pub parent_node_type: Option<Spanned<NodeType>>,
    pub allowed_node_types: Vec<NodeType>,
}

impl Diagnostic for InvalidNodeType {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            self.title_snippet().element(
                Snippet::source(source)
                    .path(path)
                    .annotation(
                        AnnotationKind::Primary.span(self.node_type.into()).label(
                            if let Some(parent_node_type) = self.parent_node_type {
                                format!(
                                    "node type can't be used as a sub-node of a {parent_node_type}",
                                )
                            } else {
                                "node type can't be used as the root".into()
                            },
                        ),
                    )
                    .annotations(self.parent_node_type.map(|pnt| {
                        AnnotationKind::Context
                            .span(pnt.span.into())
                            .label("in this node")
                    })),
            ),
            Group::with_title(Level::NOTE.secondary_title(format!(
                "valid node types are: {}",
                self.allowed_node_types.iter().join(", ")
            ))),
        ]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.node_type
    }

    fn title(&self) -> Cow<'static, str> {
        "invalid node type".into()
    }
}

#[derive(Debug)]
pub struct MissingRequiredProperty {
    pub node_type: Spanned<NodeType>,
    pub property_name: String,
    pub short: bool,
    pub allowed_property_types: Vec<String>,
    pub example_values: Vec<Cow<'static, str>>,
    pub properties_span: Option<Span>,
}

impl Diagnostic for MissingRequiredProperty {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        let label = if self.short {
            format!(
                "missing short property for `{}`, with one of these expression types: {}",
                self.property_name,
                self.allowed_property_types.join(", ")
            )
        } else {
            format!(
                "missing property `{}`, with one of these expression types: {}",
                self.property_name,
                self.allowed_property_types.join(", ")
            )
        };

        [self
            .title_snippet()
            .element(
                Snippet::source(source).path(path).annotation(
                    AnnotationKind::Primary
                        .span(self.node_type.span.into())
                        .label(label),
                ),
            )
            .elements(self.example_values.iter().flat_map(|example_value| {
                self.properties_span.map(|properties_span| {
                    Snippet::source(source).path(path).patch(Patch::new(
                        if self.short {
                            properties_span.end..properties_span.end
                        } else {
                            properties_span.start..properties_span.start
                        },
                        if self.short {
                            format!(" {example_value}")
                        } else {
                            format!("{}: {example_value},\n", self.property_name)
                        },
                    ))
                })
            }))]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.node_type.span
    }

    fn title(&self) -> Cow<'static, str> {
        format!(
            "{} node is missing a required property: {}",
            self.node_type, self.property_name
        )
        .into()
    }
}

#[derive(Debug)]
pub struct InvalidSubnode {
    pub node_type: Spanned<NodeType>,
    pub subnode: Span,
}

impl Diagnostic for InvalidSubnode {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [self.title_snippet().element(
            Snippet::source(source)
                .path(path)
                .annotation(
                    AnnotationKind::Primary
                        .span(self.subnode.into())
                        .label("subnode not supported in this location"),
                )
                .annotation(
                    AnnotationKind::Context
                        .span(self.node_type.span.into())
                        .label(format!("{} nodes don't support subnodes", self.node_type)),
                ),
        )]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.subnode
    }

    fn title(&self) -> Cow<'static, str> {
        "invalid subnode".into()
    }
}

#[derive(Debug)]
pub struct SizeBytesTooLarge {
    pub value: Span,
    pub field_set: Span,
}

impl Diagnostic for SizeBytesTooLarge {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            self.title_snippet().element(
                Snippet::source(source)
                    .path(path)
                    .annotation(AnnotationKind::Context.span(self.field_set.into()))
                    .annotation(
                        AnnotationKind::Primary
                            .span(self.value.into())
                            .label("value too large"),
                    ),
            ),
            Level::HELP
                .secondary_title(
                    "the maximum value of size-bytes is 0x10_0000 (or 1MB). Keep the value below the limit",
                )
                .element(
                    Snippet::source(source)
                        .path(path)
                        .patch(Patch::new(self.value.into(), "0xFFFF_FFFF")),
                ),
        ]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.value
    }

    fn title(&self) -> Cow<'static, str> {
        "size-bytes too large".into()
    }
}

#[derive(Debug)]
pub struct FieldAddressOutOfRange {
    pub field_address: Span,
}

impl Diagnostic for FieldAddressOutOfRange {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [self.title_snippet().element(
            Snippet::source(source).path(path).annotation(
                AnnotationKind::Primary
                    .span(self.field_address.into())
                    .label("address must be non-negative and lower than 2^32"),
            ),
        )]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.field_address
    }

    fn title(&self) -> Cow<'static, str> {
        "field address exceeds the allowed limits".into()
    }
}

#[derive(Debug)]
pub struct ResetValueNegative {
    pub reset_value: Span,
}

impl Diagnostic for ResetValueNegative {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [self.title_snippet().element(
            Snippet::source(source).path(path).annotation(
                AnnotationKind::Primary
                    .span(self.reset_value.into())
                    .label("value may not be negative"),
            ),
        )]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.reset_value
    }

    fn title(&self) -> Cow<'static, str> {
        "reset value is negative".into()
    }
}

#[derive(Debug)]
pub struct InvalidShortProperty {
    pub property: Span,
    pub node_type: Spanned<NodeType>,
    pub got: String,
    pub expected: Vec<(String, Istr)>,
}

impl Diagnostic for InvalidShortProperty {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [self.title_snippet().element(
            Snippet::source(source)
                .path(path)
                .annotation(AnnotationKind::Primary.span(self.property.into()).label(
                    if self.expected.is_empty() {
                        "no short properties are expected".into()
                    } else {
                        format!(
                            "expected one of: {}",
                            self.expected
                                .iter()
                                .map(|(expression, purpose)| format!("`{expression}` as {purpose}"))
                                .join(", ")
                        )
                    },
                ))
                .annotation(
                    AnnotationKind::Context
                        .span(self.property.into())
                        .label(format!("got: `{}`", self.got,)),
                ),
        )]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.property
    }

    fn title(&self) -> Cow<'static, str> {
        format!("invalid short property for `{}` nodes", self.node_type).into()
    }
}

#[derive(Debug)]
pub struct FieldAddressWrongOrder {
    pub address: Span,
    pub end: i128,
    pub start: i128,
}

impl Diagnostic for FieldAddressWrongOrder {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const NOTE_TEXT: &str = "the ordering is `high:low` because that mirrors the format commonly used in datasheets and HDLs";
        [
            self.title_snippet().element(
                Snippet::source(source).path(path).annotation(
                    AnnotationKind::Primary
                        .span(self.address.into())
                        .label("address must be specified as `high:low`"),
                ),
            ),
            Level::HELP
                .secondary_title("try switching around the numbers")
                .element(Snippet::source(source).path(path).patch(Patch::new(
                    self.address.into(),
                    format!("{}:{}", self.start, self.end),
                ))),
            Group::with_title(Level::NOTE.secondary_title(NOTE_TEXT)),
        ]
        .into()
    }

    fn primary_span(&self) -> Span {
        self.address
    }

    fn title(&self) -> Cow<'static, str> {
        "field address specified in wrong order".into()
    }
}

#[derive(Debug)]
pub struct IgnoredDocCommentOnProperty {
    pub doc_comments: Span,
    pub property: Span,
}

impl Diagnostic for IgnoredDocCommentOnProperty {
    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [self.title_snippet().element(
            Snippet::source(source)
                .path(path)
                .annotation(
                    AnnotationKind::Primary
                        .span(self.doc_comments.into())
                        .label("these doc comments are ignored"),
                )
                .annotation(AnnotationKind::Visible.span(self.property.into())),
        )]
        .into()
    }

    fn primary_span(&self) -> Span {
        self.doc_comments
    }

    fn title(&self) -> Cow<'static, str> {
        "doc comments placed on property that doesn't use them".into()
    }
}

#[derive(Debug)]
pub struct InvalidTypeSpecifier {
    pub node_type: Spanned<NodeType>,
    pub type_specifier: Span,
}

impl Diagnostic for InvalidTypeSpecifier {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            self.title_snippet().element(
                Snippet::source(source)
                    .path(path)
                    .annotation(AnnotationKind::Visible.span(self.node_type.span.into()))
                    .annotation(
                        AnnotationKind::Primary
                            .span(self.type_specifier.into())
                            .label("no type specifier is allowed on this node"),
                    ),
            ),
            Level::HELP
                .secondary_title("remove the type specifier")
                .element(
                    Snippet::source(source)
                        .path(path)
                        .patch(Patch::new(self.type_specifier.into(), "")),
                ),
        ]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.type_specifier
    }

    fn title(&self) -> Cow<'static, str> {
        format!("invalid type specifier for `{}` nodes", self.node_type).into()
    }
}

#[derive(Debug)]
pub struct InvalidTypeConversion {
    pub node_type: Spanned<NodeType>,
    pub type_conversion: Span,
}

impl Diagnostic for InvalidTypeConversion {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            self.title_snippet().element(
                Snippet::source(source)
                    .path(path)
                    .annotation(AnnotationKind::Visible.span(self.node_type.span.into()))
                    .annotation(
                        AnnotationKind::Primary
                            .span(self.type_conversion.into())
                            .label("no type conversion is allowed on this node"),
                    ),
            ),
            Level::HELP
                .secondary_title("remove the type conversion")
                .element(
                    Snippet::source(source)
                        .path(path)
                        .patch(Patch::new(self.type_conversion.into(), "")),
                ),
        ]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.type_conversion
    }

    fn title(&self) -> Cow<'static, str> {
        format!("invalid type conversion for `{}` nodes", self.node_type).into()
    }
}

#[derive(Debug)]
pub struct InvalidFieldsetRef {
    pub reference: Span,
    pub pointee: Option<Span>,
}

impl Diagnostic for InvalidFieldsetRef {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [self.title_snippet().element(
            Snippet::source(source)
                .path(path)
                .annotation(
                    AnnotationKind::Primary
                        .span(self.reference.into())
                        .label("no fieldset found with this name"),
                )
                .annotations(self.pointee.map(|pointee| {
                    AnnotationKind::Context
                        .span(pointee.into())
                        .label("reference points to this non-fieldset object instead")
                })),
        )]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.reference
    }

    fn title(&self) -> Cow<'static, str> {
        "invalid fieldset reference".into()
    }
}

#[derive(Debug)]
pub struct InvalidRepeat {
    pub repeat: Span,
    pub node_type: Spanned<NodeType>,
}

impl Diagnostic for InvalidRepeat {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        [
            self.title_snippet().element(
                Snippet::source(source)
                    .path(path)
                    .annotation(
                        AnnotationKind::Primary
                            .span(self.repeat.into())
                            .label(format!(
                                "repeats can't be applied on {} nodes",
                                self.node_type
                            )),
                    )
                    .annotation(AnnotationKind::Visible.span(self.node_type.span.into())),
            ),
            Level::HELP.secondary_title("remove the repeat").element(
                Snippet::source(source)
                    .path(path)
                    .patch(Patch::new(self.repeat.into(), "")),
            ),
        ]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.repeat
    }

    fn title(&self) -> Cow<'static, str> {
        "invalid repeat for node".into()
    }
}

#[derive(Debug)]
pub struct ZeroStrideRepeat {
    pub stride: Span,
}

impl Diagnostic for ZeroStrideRepeat {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const INFO_TEXT: &str = "a stride of 0 means the address doesn't change. So the repeat is useless and thus rejected";

        [
            self.title_snippet().element(
                Snippet::source(source).path(path).annotation(
                    AnnotationKind::Primary
                        .span(self.stride.into())
                        .label("stride is 0"),
                ),
            ),
            Level::HELP
                .secondary_title("change to a non-zero value")
                .element(
                    Snippet::source(source)
                        .path(path)
                        .patch(Patch::new(self.stride.into(), "1")),
                ),
            Group::with_title(Level::INFO.secondary_title(INFO_TEXT)),
        ]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.stride
    }

    fn title(&self) -> Cow<'static, str> {
        "repeat stride cannot be 0".into()
    }
}

#[derive(Debug)]
pub struct ReservedOperationNameUsed {
    pub name: Span,
    pub operation_name: String,
    pub reserved_names: &'static [&'static str],
}

impl Diagnostic for ReservedOperationNameUsed {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        let info_text: String = format!(
            "reserved names are: {}",
            self.reserved_names
                .iter()
                .map(|name| format!("`{name}`"))
                .join(", ")
        );

        [
            self.title_snippet().element(
                Snippet::source(source).path(path).annotation(
                    AnnotationKind::Primary
                        .span(self.name.into())
                        .label(format!(
                            "`{}` is a reserved name for operations. Change it to something else",
                            self.operation_name
                        )),
                ),
            ),
            Group::with_title(Level::INFO.secondary_title(info_text)),
        ]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.name
    }

    fn title(&self) -> Cow<'static, str> {
        "reserved operation name used".into()
    }
}

#[derive(Debug)]
pub struct FieldSetterNameCollision {
    pub field: Span,
    pub setter_name: String,
    pub collision_field: Span,
}

impl Diagnostic for FieldSetterNameCollision {
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn as_report<'a>(&'a self, source: &'a str, path: &'a str) -> Vec<Group<'a>> {
        const HELP_TEXT: &str = "writable fields generate setter functions that have the word `set` prepended. This can collide with other field names.\nAvoid this by changing the name of one of the fields or by making the field read only so it doesn't generate a setter";

        [
            self.title_snippet().element(
                Snippet::source(source)
                    .path(path)
                    .annotation(
                        AnnotationKind::Primary
                            .span(self.field.into())
                            .label(format!(
                                "this field is writable and generates a setter with a name that collides with another field: `{}`",
                                self.setter_name
                            )),
                    )
                    .annotation(
                        AnnotationKind::Context
                            .span(self.collision_field.into())
                            .label("collides with this field"),
                    ),
            ),
            Group::with_title(Level::HELP.secondary_title(HELP_TEXT)),
        ]
        .to_vec()
    }

    fn primary_span(&self) -> Span {
        self.field
    }

    fn title(&self) -> Cow<'static, str> {
        "field setter name collision".into()
    }
}
