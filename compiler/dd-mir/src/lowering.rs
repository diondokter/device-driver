use std::{
    borrow::Cow,
    collections::HashMap,
    mem::{self, Discriminant, discriminant},
    num::NonZero,
    str::FromStr,
    sync::LazyLock,
};

use crate::model::{
    Block, Buffer, Command, Device, Enum, EnumValue, EnumVariant, Extern, Field, FieldSet,
    Manifest, Object, Register,
};
use convert_case::Boundary;
use device_driver_common::{
    identifier::{Identifier, IdentifierRef},
    span::{Span, SpanExt, Spanned},
    specifiers::{
        Access, AddressRange, BaseType, ByteOrder, Integer, NodeType, Repeat, RepeatSource,
        ResetValue, TypeConversion,
    },
};
use device_driver_diagnostics::{
    Diagnostics,
    errors::{
        DuplicateProperty, FieldAddressOutOfRange, FieldAddressWrongOrder,
        IgnoredDocCommentOnProperty, InvalidExpressionType, InvalidIdentifier, InvalidNodeType,
        InvalidPropertyName, InvalidShortProperty, InvalidSubnode, InvalidTypeConversion,
        InvalidTypeSpecifier, MissingRequiredProperty, SizeBitsTooLarge, UnknownNodeType,
    },
};
use device_driver_parser::{Ast, Expression, Ident, Node, Property};
use itertools::Itertools;

pub fn lower(ast: Ast, diagnostics: &mut Diagnostics) -> Manifest {
    let Some(root_node) = ast.root_node else {
        return Default::default();
    };

    let result = lower_node(
        &root_node,
        None,
        &[NodeType::Manifest, NodeType::Device],
        diagnostics,
    );

    match result {
        LowerResult::Manifest(m) => m,
        LowerResult::Objects(Object::Device(d), siblings) => {
            assert!(siblings.is_empty(), "Device doesn't have sibling objects");
            d.into()
        }
        LowerResult::Objects(_, _) => unreachable!(),
        LowerResult::Error(_) => Default::default(),
    }
}

enum LowerResult {
    Manifest(Manifest),
    Objects(Object, Vec<Object>),
    Error(Vec<Object>),
}

fn lower_node(
    node: &Node,
    parent_node_type: Option<Spanned<NodeType>>,
    allowed_node_types: &[NodeType],
    diagnostics: &mut Diagnostics,
) -> LowerResult {
    let Ok(node_type) = NodeType::from_str(node.node_type.val) else {
        diagnostics.add(UnknownNodeType {
            node_type: node.node_type.span,
            allowed_node_types: allowed_node_types.to_vec(),
        });
        return LowerResult::Error(Vec::new());
    };
    let node_type = node_type.with_span(node.node_type.span);

    if !allowed_node_types.contains(&node_type) {
        diagnostics.add(InvalidNodeType {
            node_type: node_type.span,
            parent_node_type,
            allowed_node_types: allowed_node_types.to_vec(),
        });
        return LowerResult::Error(Vec::new());
    }

    match node_type.value {
        NodeType::Manifest => match parse_node_to_shape(node, diagnostics) {
            Ok((val, siblings)) => {
                assert!(siblings.is_empty(), "Manifest has no siblings");
                LowerResult::Manifest(val)
            }
            Err(siblings) => LowerResult::Error(siblings),
        },
        NodeType::Device => match parse_node_to_shape(node, diagnostics) {
            Ok((val, siblings)) => LowerResult::Objects(Object::Device(val), siblings),
            Err(siblings) => LowerResult::Error(siblings),
        },
        NodeType::Block => match parse_node_to_shape(node, diagnostics) {
            Ok((val, siblings)) => LowerResult::Objects(Object::Block(val), siblings),
            Err(siblings) => LowerResult::Error(siblings),
        },
        NodeType::Register => match parse_node_to_shape(node, diagnostics) {
            Ok((val, siblings)) => LowerResult::Objects(Object::Register(val), siblings),
            Err(siblings) => LowerResult::Error(siblings),
        },
        NodeType::Command => match parse_node_to_shape(node, diagnostics) {
            Ok((val, siblings)) => LowerResult::Objects(Object::Command(val), siblings),
            Err(siblings) => LowerResult::Error(siblings),
        },
        NodeType::Buffer => match parse_node_to_shape(node, diagnostics) {
            Ok((val, siblings)) => LowerResult::Objects(Object::Buffer(val), siblings),
            Err(siblings) => LowerResult::Error(siblings),
        },
        NodeType::FieldSet => match parse_node_to_shape(node, diagnostics) {
            Ok((val, siblings)) => LowerResult::Objects(Object::FieldSet(val), siblings),
            Err(siblings) => LowerResult::Error(siblings),
        },
        NodeType::Enum => match parse_node_to_shape(node, diagnostics) {
            Ok((val, siblings)) => LowerResult::Objects(Object::Enum(val), siblings),
            Err(siblings) => LowerResult::Error(siblings),
        },
        NodeType::Extern => match parse_node_to_shape(node, diagnostics) {
            Ok((val, siblings)) => LowerResult::Objects(Object::Extern(val), siblings),
            Err(siblings) => LowerResult::Error(siblings),
        },
        NodeType::Field => match parse_node_to_shape(node, diagnostics) {
            Ok((val, siblings)) => LowerResult::Objects(Object::Field(val), siblings),
            Err(siblings) => LowerResult::Error(siblings),
        },
    }
}

fn parse_node_to_shape<'src, S: Shape>(
    node: &Node<'src>,
    diagnostics: &mut Diagnostics,
) -> Result<(S, Vec<Object>), Vec<Object>> {
    let mut target = S::default();
    let mut sibling_objects = Vec::new();
    let mut error = false;

    // Doc comments

    *target.doc_comments() = node.doc_comments.iter().map(|c| c.value).join("\n");

    // Object name

    match Identifier::try_parse(node.name.val) {
        Ok(ident) => *target.name() = ident.with_span(node.name.span),
        Err(e) => {
            diagnostics.add(InvalidIdentifier::new(e, node.name.span));
            // Can't continue with this node when there's no name
            error = true;
        }
    }

    // Base type

    match (target.base_type(), node.type_specifier.as_ref()) {
        (None, None) => {}
        (None, Some(type_specifier)) => {
            diagnostics.add(InvalidTypeSpecifier {
                node_type: S::NODE_TYPE.with_span(node.node_type.span),
                type_specifier: type_specifier.span,
            });
        }
        (Some(base_type), None) => *base_type = BaseType::Unspecified.with_dummy_span(),
        (Some(base_type), Some(type_specifier)) => {
            *base_type = type_specifier.base_type;
        }
    }

    // Conversion

    match (target.conversion_type(), node.type_specifier.as_ref()) {
        (None, Some(type_specifier)) if type_specifier.conversion.is_some() => {
            if target.base_type().is_some() {
                // Only emit this diagnostic if a base type is supported. Otherwise we'll get double diagnostics
                diagnostics.add(InvalidTypeConversion {
                    node_type: S::NODE_TYPE.with_span(node.node_type.span),
                    type_conversion: type_specifier.span.skip(type_specifier.base_type.span),
                });
            }
        }
        (None, _) => {}
        (Some(conversion_type), None) => *conversion_type = None,
        (Some(conversion_type), Some(type_specifier)) => {
            *conversion_type = type_specifier.conversion.as_ref().and_then(|c| {
                let reference = match c {
                    device_driver_parser::TypeConversion::Reference(ident) => {
                        Some(IdentifierRef::new(ident.val.into()).with_span(ident.span))
                    }
                    device_driver_parser::TypeConversion::Subnode(sub_node) => {
                        let sub_node = lower_node(
                            sub_node,
                            Some(NodeType::Field.with_span(node.node_type.span)),
                            &[NodeType::Enum, NodeType::Extern],
                            diagnostics,
                        );

                        match sub_node {
                            LowerResult::Manifest(_) => unreachable!(),
                            LowerResult::Objects(object, objects) => {
                                let reference =
                                    object.name().take_ref().with_span(object.name_span());
                                sibling_objects.push(object);
                                sibling_objects.extend(objects);
                                Some(reference)
                            }
                            LowerResult::Error(objects) => {
                                sibling_objects.extend(objects);
                                None
                            }
                        }
                    }
                };

                reference.map(|reference| TypeConversion {
                    type_name: reference,
                    fallible: type_specifier.use_try,
                })
            })
        }
    }

    // Properties

    let mut possible_properties = S::supported_properties().to_vec();
    let mut removed_properties = HashMap::new();
    let mut removed_short_properties = HashMap::new();
    for property in &node.properties {
        let Some(property_info) = possible_properties
            .iter()
            .find(|p| p.name == PropertyName::Exact(property.name.val))
        else {
            if let Some(original) = removed_properties.get(property.name.val).copied() {
                diagnostics.add(DuplicateProperty {
                    original,
                    duplicate: property.name.span,
                });
            } else {
                diagnostics.add(InvalidPropertyName {
                    property: property.name.span,
                    node_type: S::NODE_TYPE.with_span(node.node_type.span),
                    expected_names: S::supported_properties()
                        .iter()
                        .filter_map(|p| p.name.as_exact())
                        .sorted()
                        .copied()
                        .collect(),
                });
            }

            continue;
        };

        if !property.doc_comments.is_empty() && !property_info.supports_doc_comments {
            let doc_comments = property
                .doc_comments
                .iter()
                .map(|dc| dc.span)
                .reduce(|x, y| x.to(y))
                .unwrap();

            diagnostics.add(IgnoredDocCommentOnProperty {
                doc_comments,
                property: property.name.span,
            });
        }

        // Get the discriminant and cast it to the static lifetime which is explicitly allowed in the rust docs
        let current_expression_type = unsafe {
            std::mem::transmute::<Discriminant<Expression<'src>>, Discriminant<Expression<'static>>>(
                mem::discriminant(&property.expression.value),
            )
        };

        let expression_supported =
            property_info
                .allowed_expression_types
                .iter()
                .any(|allowed_expression_type| {
                    current_expression_type == mem::discriminant(allowed_expression_type)
                });

        if !expression_supported {
            diagnostics.add(InvalidExpressionType {
                expression: property
                    .expression
                    .to_string()
                    .with_span(property.expression.span),
                node_type: S::NODE_TYPE.with_span(node.node_type.span),
                valid_expression_types: property_info
                    .allowed_expression_types
                    .iter()
                    .map(|e| e.to_string())
                    .collect(),
                valid_expression_values: property_info
                    .allowed_expression_types
                    .iter()
                    .map(|e| e.get_human_string())
                    .collect(),
            });
            continue;
        }

        error |= (property_info.setter)(
            &mut target,
            property,
            node,
            diagnostics,
            &mut sibling_objects,
        );

        if !property_info.multiple_allowed {
            possible_properties.remove(possible_properties.element_offset(property_info).unwrap());
            removed_properties.insert(property.name.val, property.name.span);
        }
    }

    for short_property in node.short_properties.iter() {
        let short_property_discriminant = discriminant(&short_property.value);

        let Some(property_info) = possible_properties.iter().find(|p| {
            p.name.as_short().is_some()
                && p.allowed_expression_types
                    .iter()
                    .map(discriminant)
                    .any(|ed| ed == short_property_discriminant)
        }) else {
            if let Some(original) = removed_short_properties
                .get(&short_property_discriminant)
                .copied()
            {
                diagnostics.add(DuplicateProperty {
                    original,
                    duplicate: short_property.span,
                });
            } else {
                diagnostics.add(InvalidShortProperty {
                    property: short_property.span,
                    node_type: S::NODE_TYPE.with_span(node.node_type.span),
                    expected: S::supported_properties()
                        .iter()
                        .filter_map(|p| {
                            p.name.as_short().map(|purpose| {
                                p.allowed_expression_types
                                    .iter()
                                    .map(|e| (e.to_string(), purpose.to_string()))
                            })
                        })
                        .flatten()
                        .sorted()
                        .collect(),
                });
            }

            continue;
        };

        error |= (property_info.setter)(
            &mut target,
            &Property {
                doc_comments: Vec::new(),
                name: Ident {
                    val: "",
                    span: short_property.span,
                },
                expression: short_property.clone(),
            }
            .with_span(short_property.span),
            node,
            diagnostics,
            &mut sibling_objects,
        );

        if !property_info.multiple_allowed {
            for allowed_expression in property_info.allowed_expression_types.iter() {
                removed_short_properties
                    .insert(discriminant(allowed_expression), short_property.span);
            }
            possible_properties.remove(possible_properties.element_offset(property_info).unwrap());
        }
    }

    // Required properties that haven't been seen
    let missing_properties = possible_properties
        .iter()
        .filter(|info| info.required)
        .collect::<Vec<_>>();

    if !missing_properties.is_empty() {
        for missing_info in missing_properties {
            diagnostics.add(MissingRequiredProperty {
                node_type: S::NODE_TYPE.with_span(node.node_type.span),
                property_name: match missing_info.name {
                    PropertyName::Exact(val) => val.to_string(),
                    PropertyName::Short(val) => val.to_string(),
                    _ => "*".to_string(),
                },
                short: matches!(missing_info.name, PropertyName::Short(_)),
                allowed_property_types: missing_info
                    .allowed_expression_types
                    .iter()
                    .map(|e| e.to_string())
                    .collect(),
            });
        }
        error = true;
    }

    // Sub nodes

    if let Some(supported_subnodes) = S::supported_subnodes() {
        for sub_node in node.sub_nodes.iter() {
            let sub_node_result = lower_node(
                sub_node,
                Some(S::NODE_TYPE.with_span(node.node_type.span)),
                supported_subnodes,
                diagnostics,
            );

            match sub_node_result {
                LowerResult::Manifest(_) => unreachable!(),
                LowerResult::Objects(object, siblings) => {
                    target.push_subnode(object);
                    sibling_objects.extend(siblings);
                }
                LowerResult::Error(siblings) => {
                    sibling_objects.extend(siblings);
                }
            }
        }
    } else if let Some(subnode) = node.sub_nodes.first() {
        diagnostics.add(InvalidSubnode {
            node_type: S::NODE_TYPE.with_span(node.node_type.span),
            subnode: subnode.span,
        });
    }

    // Make all sibling objects into subnodes if supported
    if let Some(supported_subnodes) = S::supported_subnodes() {
        for i in (0..sibling_objects.len()).rev() {
            if supported_subnodes.contains(&sibling_objects[i].node_type()) {
                target.push_subnode(sibling_objects.remove(i));
            }
        }
    }

    if !error {
        Ok((target, sibling_objects))
    } else {
        Err(sibling_objects)
    }
}

trait Shape: Default + 'static {
    const NODE_TYPE: NodeType;

    fn doc_comments(&mut self) -> &mut String;
    fn name(&mut self) -> &mut Spanned<Identifier>;

    /// All the supported properties
    fn supported_properties() -> &'static [PropertyInfo<Self>];

    fn supported_subnodes() -> Option<&'static [NodeType]> {
        None
    }

    fn push_subnode(&mut self, _: Object) {
        unimplemented!()
    }

    /// If the shape requires a base type, Some is returned
    fn base_type(&mut self) -> Option<&mut Spanned<BaseType>> {
        None
    }

    fn conversion_type(&mut self) -> Option<&mut Option<TypeConversion>> {
        None
    }
}

struct PropertyInfo<T: ?Sized> {
    name: PropertyName<'static>,
    /// The types of expressions that are supported.
    /// Comparison is done using discriminants only.
    /// The values of the expressions are used for suggestions in diagnostics.
    allowed_expression_types: Cow<'static, [Expression<'static>]>,
    /// If true, multiple of these properties are allowed
    multiple_allowed: bool,
    /// If true, the property must be set by the user.
    /// Doesn't work well with [Self::multiple_allowed] set at the same time.
    required: bool,
    /// If false, a warning is emitted when the property has doc comments
    supports_doc_comments: bool,
    /// If setter returns true, there's an error
    setter: fn(
        &mut T, // self param
        property: &Spanned<Property<'_>>,
        node: &Node, // The node that's being parsed
        diagnostics: &mut Diagnostics,
        sibling_objects: &mut Vec<Object>,
    ) -> bool,
}

impl<T: ?Sized> Clone for PropertyInfo<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            allowed_expression_types: self.allowed_expression_types.clone(),
            multiple_allowed: self.multiple_allowed,
            required: self.required,
            supports_doc_comments: self.supports_doc_comments,
            setter: self.setter,
        }
    }
}

#[derive(Clone, Copy)]
enum PropertyName<'a> {
    Exact(&'a str),
    Any,
    Short(&'a str),
}

impl<'a> PropertyName<'a> {
    fn as_exact(&self) -> Option<&&'a str> {
        if let Self::Exact(v) = self {
            Some(v)
        } else {
            None
        }
    }

    fn as_short(&self) -> Option<&&'a str> {
        if let Self::Short(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl<'a> PartialEq for PropertyName<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Exact(l0), Self::Exact(r0)) => l0 == r0,
            (Self::Short(l0), Self::Short(r0)) => l0 == r0,
            (Self::Exact(_), Self::Any) => true,
            (Self::Any, Self::Exact(_)) => true,
            _ => false,
        }
    }
}

const FIELD_SET_EXAMPLE: Node<'static> = Node {
    doc_comments: Vec::new(),
    node_type: device_driver_parser::Ident {
        val: "fieldset",
        span: Span::empty(),
    },
    name: device_driver_parser::Ident {
        val: "MyFieldSet",
        span: Span::empty(),
    },
    type_specifier: None,
    properties: Vec::new(),
    short_properties: Vec::new(),
    sub_nodes: Vec::new(),
    span: Span::empty(),
};

impl Shape for Manifest {
    const NODE_TYPE: NodeType = NodeType::Manifest;

    fn doc_comments(&mut self) -> &mut String {
        &mut self.description
    }

    fn name(&mut self) -> &mut Spanned<Identifier> {
        &mut self.name
    }

    fn supported_properties() -> &'static [PropertyInfo<Self>] {
        static MAP: &[PropertyInfo<Manifest>] = &[
            PropertyInfo {
                name: PropertyName::Exact("byte-order"),
                allowed_expression_types: Cow::Borrowed(&[Expression::ByteOrder(ByteOrder::LE)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |manifest: &mut Manifest, property, _, _, _| {
                    manifest.config.byte_order = Some(property.expression.as_byte_order().unwrap());
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("register-address-type"),
                allowed_expression_types: Cow::Borrowed(&[Expression::Integer(Integer::I32)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |manifest: &mut Manifest, property, _, _, _| {
                    manifest.config.register_address_type = Some(
                        property
                            .expression
                            .as_integer()
                            .unwrap()
                            .with_span(property.expression.span),
                    );
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("command-address-type"),
                allowed_expression_types: Cow::Borrowed(&[Expression::Integer(Integer::I32)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |manifest: &mut Manifest, property, _, _, _| {
                    manifest.config.command_address_type = Some(
                        property
                            .expression
                            .as_integer()
                            .unwrap()
                            .with_span(property.expression.span),
                    );
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("buffer-address-type"),
                allowed_expression_types: Cow::Borrowed(&[Expression::Integer(Integer::I32)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |manifest: &mut Manifest, property, _, _, _| {
                    manifest.config.buffer_address_type = Some(
                        property
                            .expression
                            .as_integer()
                            .unwrap()
                            .with_span(property.expression.span),
                    );
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("word-boundaries"),
                allowed_expression_types: Cow::Borrowed(&[Expression::String("bD:0B:_")]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |manifest: &mut Manifest, property, _, _, _| {
                    manifest.config.name_word_boundaries = Some(Boundary::defaults_from(
                        property.expression.as_string().unwrap(),
                    ));
                    false
                },
            },
        ];
        MAP
    }

    fn supported_subnodes() -> Option<&'static [NodeType]> {
        Some(&[
            NodeType::Device,
            NodeType::FieldSet,
            NodeType::Enum,
            NodeType::Extern,
        ])
    }

    fn push_subnode(&mut self, object: Object) {
        self.objects.push(object);
    }
}

impl Shape for Device {
    const NODE_TYPE: NodeType = NodeType::Device;

    fn doc_comments(&mut self) -> &mut String {
        &mut self.description
    }

    fn name(&mut self) -> &mut Spanned<Identifier> {
        &mut self.name
    }

    fn supported_properties() -> &'static [PropertyInfo<Self>] {
        static MAP: &[PropertyInfo<Device>] = &[
            PropertyInfo {
                name: PropertyName::Exact("byte-order"),
                allowed_expression_types: Cow::Borrowed(&[Expression::ByteOrder(ByteOrder::LE)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |dev: &mut Device, property, _, _, _| {
                    dev.device_config.byte_order =
                        Some(property.expression.as_byte_order().unwrap());
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("register-address-type"),
                allowed_expression_types: Cow::Borrowed(&[Expression::Integer(Integer::I32)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |dev: &mut Device, property, _, _, _| {
                    dev.device_config.register_address_type = Some(
                        property
                            .expression
                            .as_integer()
                            .unwrap()
                            .with_span(property.expression.span),
                    );
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("command-address-type"),
                allowed_expression_types: Cow::Borrowed(&[Expression::Integer(Integer::I32)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |dev: &mut Device, property, _, _, _| {
                    dev.device_config.command_address_type = Some(
                        property
                            .expression
                            .as_integer()
                            .unwrap()
                            .with_span(property.expression.span),
                    );
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("buffer-address-type"),
                allowed_expression_types: Cow::Borrowed(&[Expression::Integer(Integer::I32)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |dev: &mut Device, property, _, _, _| {
                    dev.device_config.buffer_address_type = Some(
                        property
                            .expression
                            .as_integer()
                            .unwrap()
                            .with_span(property.expression.span),
                    );
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("word-boundaries"),
                allowed_expression_types: Cow::Borrowed(&[Expression::String("bD:0B:_")]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |dev: &mut Device, property, _, _, _| {
                    dev.device_config.name_word_boundaries = Some(Boundary::defaults_from(
                        property.expression.as_string().unwrap(),
                    ));
                    false
                },
            },
        ];
        MAP
    }

    fn supported_subnodes() -> Option<&'static [NodeType]> {
        Some(&[
            NodeType::Block,
            NodeType::Register,
            NodeType::Command,
            NodeType::Buffer,
            NodeType::FieldSet,
            NodeType::Enum,
            NodeType::Extern,
        ])
    }

    fn push_subnode(&mut self, object: Object) {
        self.objects.push(object);
    }
}

impl Shape for Block {
    const NODE_TYPE: NodeType = NodeType::Block;

    fn doc_comments(&mut self) -> &mut String {
        &mut self.description
    }

    fn name(&mut self) -> &mut Spanned<Identifier> {
        &mut self.name
    }

    fn supported_properties() -> &'static [PropertyInfo<Self>] {
        static MAP: &[PropertyInfo<Block>] = &[
            PropertyInfo {
                name: PropertyName::Exact("address-offset"),
                allowed_expression_types: Cow::Borrowed(&[Expression::Number(0)]),
                multiple_allowed: false,
                required: true,
                supports_doc_comments: false,
                setter: |block: &mut Block, property, _, _, _| {
                    block.address_offset = property
                        .expression
                        .as_number()
                        .unwrap()
                        .with_span(property.expression.span);
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("repeat"),
                allowed_expression_types: Cow::Borrowed(&[Expression::Repeat(
                    device_driver_parser::Repeat {
                        source: device_driver_parser::RepeatSource::Count(NonZero::new(4).unwrap()),
                        stride: 2,
                    },
                )]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |block: &mut Block, property, _, _, _| {
                    let repeat = property.expression.as_repeat().unwrap();
                    block.repeat = Some(Repeat {
                        source: match repeat.source {
                            device_driver_parser::RepeatSource::Count(count) => {
                                RepeatSource::Count(count)
                            }
                            device_driver_parser::RepeatSource::Enum(ident) => RepeatSource::Enum(
                                IdentifierRef::new(ident.val.into()).with_span(ident.span),
                            ),
                        },
                        stride: repeat.stride as i128,
                    });
                    false
                },
            },
        ];
        MAP
    }

    fn supported_subnodes() -> Option<&'static [NodeType]> {
        Some(&[
            NodeType::Block,
            NodeType::Register,
            NodeType::Command,
            NodeType::Buffer,
            NodeType::FieldSet,
            NodeType::Enum,
            NodeType::Extern,
        ])
    }

    fn push_subnode(&mut self, object: Object) {
        self.objects.push(object);
    }
}

impl Shape for Register {
    const NODE_TYPE: NodeType = NodeType::Register;

    fn doc_comments(&mut self) -> &mut String {
        &mut self.description
    }

    fn name(&mut self) -> &mut Spanned<Identifier> {
        &mut self.name
    }

    fn supported_properties() -> &'static [PropertyInfo<Self>] {
        static MAP: LazyLock<Vec<PropertyInfo<Register>>> = LazyLock::new(|| {
            [
                PropertyInfo {
                    name: PropertyName::Exact("address"),
                    allowed_expression_types: Cow::Borrowed(&[Expression::Number(0)]),
                    multiple_allowed: false,
                    required: true,
                    supports_doc_comments: false,
                    setter: |r: &mut Register, property, _, _, _| {
                        r.address = property
                            .expression
                            .as_number()
                            .unwrap()
                            .with_span(property.expression.span);
                        false
                    },
                },
                PropertyInfo {
                    name: PropertyName::Exact("access"),
                    allowed_expression_types: Cow::Borrowed(&[Expression::Access(Access::RW)]),
                    multiple_allowed: false,
                    required: false,
                    supports_doc_comments: false,
                    setter: |r: &mut Register, property, _, _, _| {
                        r.access = property.expression.as_access().unwrap();
                        false
                    },
                },
                PropertyInfo {
                    name: PropertyName::Exact("address-overlap"),
                    allowed_expression_types: Cow::Borrowed(&[Expression::Allow]),
                    multiple_allowed: false,
                    required: false,
                    supports_doc_comments: false,
                    setter: |r: &mut Register, _, _, _, _| {
                        r.allow_address_overlap = true;
                        false
                    },
                },
                PropertyInfo {
                    name: PropertyName::Exact("reset"),
                    allowed_expression_types: Cow::Owned(vec![
                        Expression::ResetArray(vec![12, 34]),
                        Expression::ResetNumber(1234),
                    ]),
                    multiple_allowed: false,
                    required: false,
                    supports_doc_comments: false,
                    setter: |r: &mut Register, property, _, _, _| match &property.expression.value {
                        Expression::ResetNumber(num) => {
                            r.reset_value =
                                Some(ResetValue::Integer(*num).with_span(property.expression.span));
                            false
                        }
                        Expression::ResetArray(bytes) => {
                            r.reset_value = Some(
                                ResetValue::Array(bytes.to_vec())
                                    .with_span(property.expression.span),
                            );
                            false
                        }
                        _ => unreachable!(),
                    },
                },
                PropertyInfo {
                    name: PropertyName::Exact("repeat"),
                    allowed_expression_types: Cow::Borrowed(&[Expression::Repeat(
                        device_driver_parser::Repeat {
                            source: device_driver_parser::RepeatSource::Count(
                                const { NonZero::new(4).unwrap() },
                            ),
                            stride: 2,
                        },
                    )]),
                    multiple_allowed: false,
                    required: false,
                    supports_doc_comments: false,
                    setter: |r: &mut Register, property, _, _, _| {
                        let repeat = property.expression.as_repeat().unwrap();
                        r.repeat = Some(Repeat {
                            source: match repeat.source {
                                device_driver_parser::RepeatSource::Count(count) => {
                                    RepeatSource::Count(count)
                                }
                                device_driver_parser::RepeatSource::Enum(ident) => {
                                    RepeatSource::Enum(
                                        IdentifierRef::new(ident.val.into()).with_span(ident.span),
                                    )
                                }
                            },
                            stride: repeat.stride as i128,
                        });
                        false
                    },
                },
                PropertyInfo {
                    name: PropertyName::Exact("fields"),
                    allowed_expression_types: Cow::Owned(vec![
                        Expression::TypeReference(device_driver_parser::Ident {
                            val: "MyFieldset",
                            span: Span::empty(),
                        }),
                        Expression::SubNode(Box::new(FIELD_SET_EXAMPLE)),
                    ]),
                    multiple_allowed: false,
                    required: true,
                    supports_doc_comments: false,
                    setter: |r: &mut Register, property, node, diagnostics, sibling_objects| {
                        match &property.expression.value {
                            Expression::TypeReference(ident) => {
                                r.field_set_ref =
                                    IdentifierRef::new(ident.val.into()).with_span(ident.span);
                                false
                            }
                            Expression::SubNode(sub_node) => {
                                let result = lower_node(
                                    sub_node,
                                    Some(NodeType::Register.with_span(node.node_type.span)),
                                    &[NodeType::FieldSet],
                                    diagnostics,
                                );

                                match result {
                                    LowerResult::Objects(fs, fs_siblings) => {
                                        r.field_set_ref =
                                            fs.name().take_ref().with_span(fs.name_span());
                                        sibling_objects.push(fs);
                                        sibling_objects.extend(fs_siblings);
                                        false
                                    }
                                    LowerResult::Error(fs_siblings) => {
                                        sibling_objects.extend(fs_siblings);
                                        true
                                    }
                                    LowerResult::Manifest(_) => unreachable!(),
                                }
                            }
                            _ => unreachable!(),
                        }
                    },
                },
            ]
            .into()
        });
        &MAP
    }
}

impl Shape for FieldSet {
    const NODE_TYPE: NodeType = NodeType::FieldSet;

    fn doc_comments(&mut self) -> &mut String {
        &mut self.description
    }

    fn name(&mut self) -> &mut Spanned<Identifier> {
        &mut self.name
    }

    fn supported_properties() -> &'static [PropertyInfo<Self>] {
        static MAP: &[PropertyInfo<FieldSet>] = &[
            PropertyInfo {
                name: PropertyName::Exact("size-bits"),
                allowed_expression_types: Cow::Borrowed(&[Expression::Number(8)]),
                multiple_allowed: false,
                required: true,
                supports_doc_comments: false,
                setter: |fs: &mut FieldSet, property, fs_node, diagnostics, _| match u32::try_from(
                    property.expression.as_number().unwrap(),
                ) {
                    Ok(size_bits) => {
                        fs.size_bits = size_bits.with_span(property.expression.span);
                        false
                    }
                    Err(_) => {
                        diagnostics.add(SizeBitsTooLarge {
                            value: property.expression.span,
                            field_set: fs_node.span,
                        });
                        true
                    }
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("byte-order"),
                allowed_expression_types: Cow::Borrowed(&[Expression::ByteOrder(ByteOrder::LE)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |fs: &mut FieldSet, property, _, _, _| {
                    fs.byte_order = Some(property.expression.as_byte_order().unwrap());
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("bit-overlap"),
                allowed_expression_types: Cow::Borrowed(&[Expression::Allow]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |fs: &mut FieldSet, _, _, _, _| {
                    fs.allow_bit_overlap = true;
                    false
                },
            },
        ];
        MAP
    }

    fn supported_subnodes() -> Option<&'static [NodeType]> {
        Some(&[NodeType::Field])
    }

    fn push_subnode(&mut self, object: Object) {
        let Object::Field(field) = object else {
            unreachable!("{object:?}")
        };
        self.fields.push(field);
    }
}

impl Shape for Extern {
    const NODE_TYPE: NodeType = NodeType::Extern;

    fn doc_comments(&mut self) -> &mut String {
        &mut self.description
    }

    fn name(&mut self) -> &mut Spanned<Identifier> {
        &mut self.name
    }

    fn supported_properties() -> &'static [PropertyInfo<Self>] {
        static MAP: &[PropertyInfo<Extern>] = &[PropertyInfo {
            name: PropertyName::Exact("infallible"),
            allowed_expression_types: Cow::Borrowed(&[Expression::Allow]),
            multiple_allowed: false,
            required: false,
            supports_doc_comments: false,
            setter: |ext: &mut Extern, _, _, _, _| {
                ext.supports_infallible = true;
                false
            },
        }];
        MAP
    }

    fn base_type(&mut self) -> Option<&mut Spanned<BaseType>> {
        Some(&mut self.base_type)
    }
}

impl Shape for Buffer {
    const NODE_TYPE: NodeType = NodeType::Buffer;

    fn doc_comments(&mut self) -> &mut String {
        &mut self.description
    }

    fn name(&mut self) -> &mut Spanned<Identifier> {
        &mut self.name
    }

    fn supported_properties() -> &'static [PropertyInfo<Self>] {
        static MAP: &[PropertyInfo<Buffer>] = &[
            PropertyInfo {
                name: PropertyName::Exact("access"),
                allowed_expression_types: Cow::Borrowed(&[Expression::Access(Access::RW)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |buf: &mut Buffer, property, _, _, _| {
                    buf.access = property.expression.as_access().unwrap();
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("address"),
                allowed_expression_types: Cow::Borrowed(&[Expression::Number(0)]),
                multiple_allowed: false,
                required: true,
                supports_doc_comments: false,
                setter: |buf: &mut Buffer, property, _, _, _| {
                    buf.address = property
                        .expression
                        .as_number()
                        .unwrap()
                        .with_span(property.expression.span);
                    false
                },
            },
        ];
        MAP
    }
}

impl Shape for Enum {
    const NODE_TYPE: NodeType = NodeType::Enum;

    fn doc_comments(&mut self) -> &mut String {
        &mut self.description
    }

    fn name(&mut self) -> &mut Spanned<Identifier> {
        &mut self.name
    }

    fn supported_properties() -> &'static [PropertyInfo<Self>] {
        static MAP: &[PropertyInfo<Enum>] = &[PropertyInfo {
            name: PropertyName::Any,
            allowed_expression_types: Cow::Borrowed(&[
                Expression::Auto,
                Expression::Number(0),
                Expression::DefaultNumber(Some(0)),
                Expression::DefaultNumber(None),
                Expression::CatchAllNumber(Some(0)),
                Expression::CatchAllNumber(None),
            ]),
            multiple_allowed: true,
            required: false,
            supports_doc_comments: true,
            setter: |enum_value: &mut Enum, property, _, diagnostics, _| {
                let identifier = match Identifier::try_parse(property.name.val) {
                    Ok(identifier) => identifier,
                    Err(e) => {
                        diagnostics.add(InvalidIdentifier {
                            error: e,
                            identifier: property.name.span,
                        });
                        return true;
                    }
                };

                enum_value.variants.push(EnumVariant {
                    description: property.doc_comments.iter().map(|c| c.value).join("\n"),
                    name: identifier.with_span(property.name.span),
                    value: match &property.expression.value {
                        Expression::Number(num) => EnumValue::Specified(*num),
                        Expression::DefaultNumber(Some(num)) => EnumValue::Default(*num),
                        Expression::DefaultNumber(None) => EnumValue::UnspecifiedDefault,
                        Expression::CatchAllNumber(Some(num)) => EnumValue::CatchAll(*num),
                        Expression::CatchAllNumber(None) => EnumValue::UnspecifiedCatchAll,
                        Expression::Auto => EnumValue::Unspecified,
                        _ => unreachable!(),
                    },
                    span: property.span,
                });
                false
            },
        }];
        MAP
    }

    fn base_type(&mut self) -> Option<&mut Spanned<BaseType>> {
        Some(&mut self.base_type)
    }
}

impl Shape for Command {
    const NODE_TYPE: NodeType = NodeType::Command;

    fn doc_comments(&mut self) -> &mut String {
        &mut self.description
    }

    fn name(&mut self) -> &mut Spanned<Identifier> {
        &mut self.name
    }

    fn supported_properties() -> &'static [PropertyInfo<Self>] {
        static MAP: LazyLock<Vec<PropertyInfo<Command>>> = LazyLock::new(|| {
            [
                PropertyInfo {
                    name: PropertyName::Exact("address"),
                    allowed_expression_types: Cow::Borrowed(&[Expression::Number(0)]),
                    multiple_allowed: false,
                    required: true,
                    supports_doc_comments: false,
                    setter: |command: &mut Command, property, _, _, _| {
                        command.address = property
                            .expression
                            .as_number()
                            .unwrap()
                            .with_span(property.expression.span);
                        false
                    },
                },
                PropertyInfo {
                    name: PropertyName::Exact("address-overlap"),
                    allowed_expression_types: Cow::Borrowed(&[Expression::Allow]),
                    multiple_allowed: false,
                    required: false,
                    supports_doc_comments: false,
                    setter: |command: &mut Command, _, _, _, _| {
                        command.allow_address_overlap = true;
                        false
                    },
                },
                PropertyInfo {
                    name: PropertyName::Exact("repeat"),
                    allowed_expression_types: Cow::Borrowed(&[Expression::Repeat(
                        device_driver_parser::Repeat {
                            source: device_driver_parser::RepeatSource::Count(
                                const { NonZero::new(4).unwrap() },
                            ),
                            stride: 2,
                        },
                    )]),
                    multiple_allowed: false,
                    required: false,
                    supports_doc_comments: false,
                    setter: |command: &mut Command, property, _, _, _| {
                        let repeat = property.expression.as_repeat().unwrap();
                        command.repeat = Some(Repeat {
                            source: match repeat.source {
                                device_driver_parser::RepeatSource::Count(count) => {
                                    RepeatSource::Count(count)
                                }
                                device_driver_parser::RepeatSource::Enum(ident) => {
                                    RepeatSource::Enum(
                                        IdentifierRef::new(ident.val.into()).with_span(ident.span),
                                    )
                                }
                            },
                            stride: repeat.stride as i128,
                        });
                        false
                    },
                },
                PropertyInfo {
                    name: PropertyName::Exact("fields-in"),
                    allowed_expression_types: Cow::Owned(vec![
                        Expression::TypeReference(device_driver_parser::Ident {
                            val: "MyFieldset",
                            span: Span::empty(),
                        }),
                        Expression::SubNode(Box::new(FIELD_SET_EXAMPLE)),
                    ]),
                    multiple_allowed: false,
                    required: false,
                    supports_doc_comments: false,
                    setter: |command: &mut Command,
                             property,
                             node,
                             diagnostics,
                             sibling_objects| {
                        match &property.expression.value {
                            Expression::TypeReference(ident) => {
                                command.field_set_ref_in = Some(
                                    IdentifierRef::new(ident.val.into()).with_span(ident.span),
                                );
                                false
                            }
                            Expression::SubNode(sub_node) => {
                                let result = lower_node(
                                    sub_node,
                                    Some(NodeType::Register.with_span(node.node_type.span)),
                                    &[NodeType::FieldSet],
                                    diagnostics,
                                );

                                match result {
                                    LowerResult::Objects(fs, fs_siblings) => {
                                        command.field_set_ref_in =
                                            Some(fs.name().take_ref().with_span(fs.name_span()));
                                        sibling_objects.push(fs);
                                        sibling_objects.extend(fs_siblings);
                                        false
                                    }
                                    LowerResult::Error(fs_siblings) => {
                                        sibling_objects.extend(fs_siblings);
                                        true
                                    }
                                    LowerResult::Manifest(_) => unreachable!(),
                                }
                            }
                            _ => unreachable!(),
                        }
                    },
                },
                PropertyInfo {
                    name: PropertyName::Exact("fields-out"),
                    allowed_expression_types: Cow::Owned(vec![
                        Expression::TypeReference(device_driver_parser::Ident {
                            val: "MyFieldset",
                            span: Span::empty(),
                        }),
                        Expression::SubNode(Box::new(FIELD_SET_EXAMPLE)),
                    ]),
                    multiple_allowed: false,
                    required: false,
                    supports_doc_comments: false,
                    setter: |command: &mut Command,
                             property,
                             node,
                             diagnostics,
                             sibling_objects| {
                        match &property.expression.value {
                            Expression::TypeReference(ident) => {
                                command.field_set_ref_out = Some(
                                    IdentifierRef::new(ident.val.into()).with_span(ident.span),
                                );
                                false
                            }
                            Expression::SubNode(sub_node) => {
                                let result = lower_node(
                                    sub_node,
                                    Some(NodeType::Register.with_span(node.node_type.span)),
                                    &[NodeType::FieldSet],
                                    diagnostics,
                                );

                                match result {
                                    LowerResult::Objects(fs, fs_siblings) => {
                                        command.field_set_ref_out =
                                            Some(fs.name().take_ref().with_span(fs.name_span()));
                                        sibling_objects.push(fs);
                                        sibling_objects.extend(fs_siblings);
                                        false
                                    }
                                    LowerResult::Error(fs_siblings) => {
                                        sibling_objects.extend(fs_siblings);
                                        true
                                    }
                                    LowerResult::Manifest(_) => unreachable!(),
                                }
                            }
                            _ => unreachable!(),
                        }
                    },
                },
            ]
            .into()
        });
        &MAP
    }
}

impl Shape for Field {
    const NODE_TYPE: NodeType = NodeType::Field;

    fn doc_comments(&mut self) -> &mut String {
        &mut self.description
    }

    fn name(&mut self) -> &mut Spanned<Identifier> {
        &mut self.name
    }

    fn supported_properties() -> &'static [PropertyInfo<Self>] {
        static MAP: &[PropertyInfo<Field>] = &[
            PropertyInfo {
                name: PropertyName::Short("address"),
                allowed_expression_types: Cow::Borrowed(&[
                    Expression::Number(0),
                    Expression::AddressRange { end: 8, start: 0 },
                ]),
                multiple_allowed: false,
                required: true,
                supports_doc_comments: false,
                setter: |field: &mut Field, property, _, diagnostics, _| {
                    let u32_range = 0..=u32::MAX as i128;

                    field.field_address = match property.expression.value {
                        Expression::AddressRange { end, start }
                            if u32_range.contains(&end) && u32_range.contains(&start) =>
                        {
                            if end < start {
                                diagnostics.add(FieldAddressWrongOrder {
                                    address: property.expression.span,
                                    end,
                                    start,
                                });
                                return true;
                            }

                            AddressRange {
                                start: start.try_into().unwrap(),
                                end: end.try_into().unwrap(),
                            }
                        }
                        Expression::Number(num) if u32_range.contains(&num) => AddressRange {
                            start: num.try_into().unwrap(),
                            end: num.try_into().unwrap(),
                        },
                        Expression::AddressRange { .. } | Expression::Number(_) => {
                            diagnostics.add(FieldAddressOutOfRange {
                                field_address: property.expression.span,
                            });
                            return true;
                        }
                        _ => unreachable!(),
                    }
                    .with_span(property.expression.span);
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Short("access"),
                allowed_expression_types: Cow::Borrowed(&[Expression::Access(Access::RW)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |field: &mut Field, property, _, _, _| {
                    field.access = property.expression.as_access().unwrap();
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Short("repeat"),
                allowed_expression_types: Cow::Borrowed(&[Expression::Repeat(
                    device_driver_parser::Repeat {
                        source: device_driver_parser::RepeatSource::Count(
                            const { NonZero::new(4).unwrap() },
                        ),
                        stride: 2,
                    },
                )]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |field: &mut Field, property, _, _, _| {
                    let repeat = property.expression.as_repeat().unwrap();
                    field.repeat = Some(Repeat {
                        source: match repeat.source {
                            device_driver_parser::RepeatSource::Count(count) => {
                                RepeatSource::Count(count)
                            }
                            device_driver_parser::RepeatSource::Enum(ident) => RepeatSource::Enum(
                                IdentifierRef::new(ident.val.into()).with_span(ident.span),
                            ),
                        },
                        stride: repeat.stride as i128,
                    });
                    false
                },
            },
        ];
        MAP
    }

    fn base_type(&mut self) -> Option<&mut Spanned<BaseType>> {
        Some(&mut self.base_type)
    }

    fn conversion_type(&mut self) -> Option<&mut Option<TypeConversion>> {
        Some(&mut self.field_conversion)
    }
}
