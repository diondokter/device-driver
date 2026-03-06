use std::{collections::HashMap, mem, str::FromStr, sync::OnceLock};

use crate::model::{
    Block, Buffer, Command, Device, Enum, EnumValue, EnumVariant, Extern, Field, FieldSet,
    Manifest, Object, Register,
};
use device_driver_common::{
    identifier::{Identifier, IdentifierRef},
    span::{Span, SpanExt, Spanned},
    specifiers::{AddressRange, BaseType, ByteOrder, NodeType, Repeat, RepeatSource, ResetValue},
};
use device_driver_diagnostics::{
    Diagnostics,
    errors::{
        DuplicateProperty, FieldAddressOutOfRange, InvalidExpressionType, InvalidIdentifier,
        InvalidNodeType, InvalidPropertyName, InvalidSubnode, MissingRequiredProperty,
        SizeBitsTooLarge, UnknownNodeType,
    },
};
use device_driver_parser::{Ast, Expression, Node, Property};
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

fn parse_node_to_shape<S: Shape>(
    node: &Node,
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
        (None, Some(_)) => {
            todo!("Emit diagnostic: Type specifier found for node type that doesn't support it")
        }
        (Some(base_type), None) => *base_type = BaseType::Unspecified.with_dummy_span(),
        (Some(base_type), Some(type_specifier)) => {
            *base_type = type_specifier.base_type;
        }
    }

    // Conversion: TODO

    // Properties

    let mut possible_properties = (*S::supported_properties()).clone();
    let mut removed_possible_properties = HashMap::new();
    for property in &node.properties {
        let Some(property_info) = possible_properties
            .get(property.name.val)
            .or_else(|| possible_properties.get("*"))
        else {
            if let Some(original) = removed_possible_properties.get(property.name.val).copied() {
                diagnostics.add(DuplicateProperty {
                    original,
                    duplicate: property.name.span,
                });
            } else {
                diagnostics.add(InvalidPropertyName {
                    property: property.name.span,
                    node_type: S::NODE_TYPE.with_span(node.node_type.span),
                    expected_names: S::supported_properties().keys().sorted().copied().collect(),
                });
            }

            continue;
        };

        if !property.doc_comments.is_empty() && !property_info.supports_doc_comments {
            todo!(
                "Emit diagnostic warning: Doc comments placed on property that doesn't support it"
            );
        }

        let current_expression_type = mem::discriminant(&property.expression.value);

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
            possible_properties.remove(property.name.val).unwrap();

            removed_possible_properties.insert(property.name.val, property.name.span);
        }
    }

    // Required properties that haven't been seen
    let missing_properties = possible_properties
        .iter()
        .filter(|(_, info)| info.required)
        .collect::<Vec<_>>();

    if !missing_properties.is_empty() {
        for (missing_name, missing_info) in missing_properties {
            diagnostics.add(MissingRequiredProperty {
                node_type: S::NODE_TYPE.with_span(node.node_type.span),
                property_name: missing_name.to_string(),
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
                    for sibling in siblings {
                        target.push_subnode(sibling);
                    }
                }
                LowerResult::Error(siblings) => {
                    for sibling in siblings {
                        target.push_subnode(sibling);
                    }
                }
            }
        }
    } else if let Some(subnode) = node.sub_nodes.first() {
        diagnostics.add(InvalidSubnode {
            node_type: S::NODE_TYPE.with_span(node.node_type.span),
            subnode: subnode.span,
        });
    }

    if !error {
        Ok((target, sibling_objects))
    } else {
        Err(sibling_objects)
    }
}

type Properties<S> = HashMap<&'static str, PropertyInfo<S>>;

trait Shape: Default + 'static {
    const NODE_TYPE: NodeType;

    fn doc_comments(&mut self) -> &mut String;
    fn name(&mut self) -> &mut Spanned<Identifier>;

    /// All the supported properties. A "*" as key matches anything
    fn supported_properties() -> &'static Properties<Self>;

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
}

struct PropertyInfo<T: ?Sized> {
    /// The types of expressions that are supported.
    /// Comparison is done using discriminants only.
    /// The values of the expressions are used for suggestions in diagnostics.
    allowed_expression_types: Vec<Expression<'static>>,
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
            allowed_expression_types: self.allowed_expression_types.clone(),
            multiple_allowed: self.multiple_allowed,
            required: self.required,
            supports_doc_comments: self.supports_doc_comments,
            setter: self.setter,
        }
    }
}

impl Shape for Manifest {
    const NODE_TYPE: NodeType = NodeType::Manifest;

    fn doc_comments(&mut self) -> &mut String {
        &mut self.description
    }

    fn name(&mut self) -> &mut Spanned<Identifier> {
        &mut self.name
    }

    fn supported_properties() -> &'static Properties<Self> {
        static MAP: OnceLock<Properties<Manifest>> = OnceLock::new();
        MAP.get_or_init(|| {
            [
                (
                    "byte-order",
                    PropertyInfo {
                        allowed_expression_types: vec![Expression::ByteOrder(Default::default())],
                        multiple_allowed: false,
                        required: false,
                        supports_doc_comments: false,
                        setter: |manifest: &mut Self, property, _, _, _| {
                            manifest.config.byte_order =
                                Some(property.expression.as_byte_order().unwrap());
                            false
                        },
                    },
                ),
                (
                    "register-address-type",
                    PropertyInfo {
                        allowed_expression_types: vec![Expression::Integer(Default::default())],
                        multiple_allowed: false,
                        required: false,
                        supports_doc_comments: false,
                        setter: |manifest: &mut Self, property, _, _, _| {
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
                ),
                (
                    "command-address-type",
                    PropertyInfo {
                        allowed_expression_types: vec![Expression::Integer(Default::default())],
                        multiple_allowed: false,
                        required: false,
                        supports_doc_comments: false,
                        setter: |manifest: &mut Self, property, _, _, _| {
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
                ),
                (
                    "buffer-address-type",
                    PropertyInfo {
                        allowed_expression_types: vec![Expression::Integer(Default::default())],
                        multiple_allowed: false,
                        required: false,
                        supports_doc_comments: false,
                        setter: |manifest: &mut Self, property, _, _, _| {
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
                ),
                // TODO: name-word-boundaries
                // TODO: defmt-feature
            ]
            .into()
        })
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

    fn supported_properties() -> &'static Properties<Self> {
        static MAP: OnceLock<Properties<Device>> = OnceLock::new();
        MAP.get_or_init(|| {
            [
                (
                    "byte-order",
                    PropertyInfo {
                        allowed_expression_types: vec![Expression::ByteOrder(Default::default())],
                        multiple_allowed: false,
                        required: false,
                        supports_doc_comments: false,
                        setter: |dev: &mut Self, property, _, _, _| {
                            dev.device_config.byte_order =
                                Some(property.expression.as_byte_order().unwrap());
                            false
                        },
                    },
                ),
                (
                    "register-address-type",
                    PropertyInfo {
                        allowed_expression_types: vec![Expression::Integer(Default::default())],
                        multiple_allowed: false,
                        required: false,
                        supports_doc_comments: false,
                        setter: |dev: &mut Self, property, _, _, _| {
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
                ),
                (
                    "command-address-type",
                    PropertyInfo {
                        allowed_expression_types: vec![Expression::Integer(Default::default())],
                        multiple_allowed: false,
                        required: false,
                        supports_doc_comments: false,
                        setter: |dev: &mut Self, property, _, _, _| {
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
                ),
                (
                    "buffer-address-type",
                    PropertyInfo {
                        allowed_expression_types: vec![Expression::Integer(Default::default())],
                        multiple_allowed: false,
                        required: false,
                        supports_doc_comments: false,
                        setter: |dev: &mut Self, property, _, _, _| {
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
                ),
                // TODO: name-word-boundaries
                // TODO: defmt-feature
            ]
            .into()
        })
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

    fn supported_properties() -> &'static Properties<Self> {
        static MAP: OnceLock<Properties<Block>> = OnceLock::new();
        MAP.get_or_init(|| {
            [
                (
                    "address-offset",
                    PropertyInfo {
                        allowed_expression_types: vec![Expression::Number(Default::default())],
                        multiple_allowed: false,
                        required: true,
                        supports_doc_comments: false,
                        setter: |block: &mut Self, property, _, _, _| {
                            block.address_offset = property
                                .expression
                                .as_number()
                                .unwrap()
                                .with_span(property.expression.span);
                            false
                        },
                    },
                ),
                (
                    "repeat",
                    PropertyInfo {
                        allowed_expression_types: vec![Expression::Repeat(Default::default())],
                        multiple_allowed: false,
                        required: false,
                        supports_doc_comments: false,
                        setter: |block: &mut Self, property, _, _, _| {
                            block.address_offset = property
                                .expression
                                .as_number()
                                .unwrap()
                                .with_span(property.expression.span);
                            false
                        },
                    },
                ),
            ]
            .into()
        })
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

    fn supported_properties() -> &'static Properties<Self> {
        static MAP: OnceLock<Properties<Register>> = OnceLock::new();
        MAP.get_or_init(|| {
            [
                (
                    "address",
                    PropertyInfo {
                        allowed_expression_types: vec![Expression::Number(Default::default())],
                        multiple_allowed: false,
                        required: true,
                        supports_doc_comments: false,
                        setter: |r: &mut Self, property, _, _, _| {
                            r.address = property
                                .expression
                                .as_number()
                                .unwrap()
                                .with_span(property.expression.span);
                            false
                        },
                    },
                ),
                (
                    "access",
                    PropertyInfo {
                        allowed_expression_types: vec![Expression::Access(Default::default())],
                        multiple_allowed: false,
                        required: false,
                        supports_doc_comments: false,
                        setter: |r: &mut Self, property, _, _, _| {
                            r.access = property.expression.as_access().unwrap();
                            false
                        },
                    },
                ),
                (
                    "address-overlap",
                    PropertyInfo {
                        allowed_expression_types: vec![Expression::Allow],
                        multiple_allowed: false,
                        required: false,
                        supports_doc_comments: false,
                        setter: |r: &mut Self, _, _, _, _| {
                            r.allow_address_overlap = true;
                            false
                        },
                    },
                ),
                (
                    "reset",
                    PropertyInfo {
                        allowed_expression_types: vec![
                            Expression::ResetArray(vec![12, 34]),
                            Expression::ResetNumber(1234),
                        ],
                        multiple_allowed: false,
                        required: false,
                        supports_doc_comments: false,
                        setter: |r: &mut Self, property, _, _, _| match &property.expression.value {
                            Expression::ResetNumber(num) => {
                                r.reset_value = Some(
                                    ResetValue::Integer(*num).with_span(property.expression.span),
                                );
                                false
                            }
                            Expression::ResetArray(bytes) => {
                                r.reset_value = Some(
                                    ResetValue::Array(bytes.clone())
                                        .with_span(property.expression.span),
                                );
                                false
                            }
                            _ => unreachable!(),
                        },
                    },
                ),
                (
                    "repeat",
                    PropertyInfo {
                        allowed_expression_types: vec![Expression::Repeat(Default::default())],
                        multiple_allowed: false,
                        required: false,
                        supports_doc_comments: false,
                        setter: |r: &mut Self, property, _, _, _| {
                            let repeat = property.expression.as_repeat().unwrap();
                            r.repeat = Some(Repeat {
                                source: match repeat.source {
                                    device_driver_parser::RepeatSource::Count(count) => {
                                        RepeatSource::Count(count as u64)
                                    }
                                    device_driver_parser::RepeatSource::Enum(ident) => {
                                        RepeatSource::Enum(
                                            IdentifierRef::new(ident.val.into())
                                                .with_span(ident.span),
                                        )
                                    }
                                },
                                stride: repeat.stride as i128,
                            });
                            false
                        },
                    },
                ),
                (
                    "fields",
                    PropertyInfo {
                        allowed_expression_types: vec![
                            Expression::TypeReference(device_driver_parser::Ident {
                                val: "MyFieldset",
                                span: Span::default(),
                            }),
                            Expression::SubNode(Box::new(Node {
                                doc_comments: vec![],
                                node_type: device_driver_parser::Ident {
                                    val: "fieldset",
                                    span: Default::default(),
                                },
                                name: device_driver_parser::Ident {
                                    val: "MyFieldSet",
                                    span: Default::default(),
                                },
                                type_specifier: None,
                                properties: vec![],
                                short_properties: vec![],
                                sub_nodes: vec![],
                                span: Default::default(),
                            })),
                        ],
                        multiple_allowed: false,
                        required: true,
                        supports_doc_comments: false,
                        setter: |r: &mut Self, property, node, diagnostics, sibling_objects| {
                            match &property.expression.value {
                                Expression::TypeReference(ident) => {
                                    r.field_set_ref = IdentifierRef::new(ident.val.into());
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
                                            r.field_set_ref = fs.name().take_ref();
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
                ),
            ]
            .into()
        })
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

    fn supported_properties() -> &'static Properties<Self> {
        static MAP: OnceLock<Properties<FieldSet>> = OnceLock::new();
        MAP.get_or_init(|| {
            [
                (
                    "size-bits",
                    PropertyInfo {
                        allowed_expression_types: vec![Expression::Number(8)],
                        multiple_allowed: false,
                        required: true,
                        supports_doc_comments: false,
                        setter: |fs: &mut Self, property, fs_node, diagnostics, _| {
                            match u32::try_from(property.expression.as_number().unwrap()) {
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
                            }
                        },
                    },
                ),
                (
                    "byte-order",
                    PropertyInfo {
                        allowed_expression_types: vec![Expression::ByteOrder(ByteOrder::LE)],
                        multiple_allowed: false,
                        required: false,
                        supports_doc_comments: false,
                        setter: |fs: &mut Self, property, _, _, _| {
                            fs.byte_order = Some(property.expression.as_byte_order().unwrap());
                            false
                        },
                    },
                ),
                (
                    "bit-overlap",
                    PropertyInfo {
                        allowed_expression_types: vec![Expression::Allow],
                        multiple_allowed: false,
                        required: false,
                        supports_doc_comments: false,
                        setter: |fs: &mut Self, _, _, _, _| {
                            fs.allow_bit_overlap = true;
                            false
                        },
                    },
                ),
            ]
            .into()
        })
    }

    fn supported_subnodes() -> Option<&'static [NodeType]> {
        Some(&[NodeType::Field])
    }

    fn push_subnode(&mut self, object: Object) {
        let Object::Field(field) = object else {
            unreachable!()
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

    fn supported_properties() -> &'static Properties<Self> {
        static MAP: OnceLock<Properties<Extern>> = OnceLock::new();
        MAP.get_or_init(|| {
            [(
                "infallible",
                PropertyInfo {
                    allowed_expression_types: vec![Expression::Allow],
                    multiple_allowed: false,
                    required: false,
                    supports_doc_comments: false,
                    setter: |ext: &mut Self, _, _, _, _| {
                        ext.supports_infallible = true;
                        false
                    },
                },
            )]
            .into()
        })
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

    fn supported_properties() -> &'static Properties<Self> {
        static MAP: OnceLock<Properties<Buffer>> = OnceLock::new();
        MAP.get_or_init(|| {
            [
                (
                    "access",
                    PropertyInfo {
                        allowed_expression_types: vec![Expression::Access(Default::default())],
                        multiple_allowed: false,
                        required: false,
                        supports_doc_comments: false,
                        setter: |buf: &mut Self, property, _, _, _| {
                            buf.access = property.expression.as_access().unwrap();
                            false
                        },
                    },
                ),
                (
                    "address",
                    PropertyInfo {
                        allowed_expression_types: vec![Expression::Number(Default::default())],
                        multiple_allowed: false,
                        required: true,
                        supports_doc_comments: false,
                        setter: |buf: &mut Self, property, _, _, _| {
                            buf.address = property
                                .expression
                                .as_number()
                                .unwrap()
                                .with_span(property.expression.span);
                            false
                        },
                    },
                ),
            ]
            .into()
        })
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

    fn supported_properties() -> &'static Properties<Self> {
        static MAP: OnceLock<Properties<Enum>> = OnceLock::new();
        MAP.get_or_init(|| {
            [(
                "*",
                PropertyInfo {
                    allowed_expression_types: vec![
                        Expression::Auto,
                        Expression::Number(0),
                        Expression::DefaultNumber(0),
                        Expression::CatchAllNumber(0),
                    ],
                    multiple_allowed: true,
                    required: false,
                    supports_doc_comments: true,
                    setter: |enum_value: &mut Self, property, _, diagnostics, _| {
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
                                Expression::DefaultNumber(num) => EnumValue::Default(*num),
                                Expression::CatchAllNumber(num) => EnumValue::CatchAll(*num),
                                Expression::Auto => EnumValue::Unspecified,
                                _ => unreachable!(),
                            },
                            span: property.span,
                        });
                        false
                    },
                },
            )]
            .into()
        })
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

    fn supported_properties() -> &'static Properties<Self> {
        static MAP: OnceLock<Properties<Command>> = OnceLock::new();
        MAP.get_or_init(|| {
            [
                (
                    "address",
                    PropertyInfo {
                        allowed_expression_types: vec![Expression::Number(0)],
                        multiple_allowed: false,
                        required: true,
                        supports_doc_comments: false,
                        setter: |command: &mut Self, property, _, _, _| {
                            command.address = property
                                .expression
                                .as_number()
                                .unwrap()
                                .with_span(property.expression.span);
                            false
                        },
                    },
                ),
                (
                    "address-overlap",
                    PropertyInfo {
                        allowed_expression_types: vec![Expression::Allow],
                        multiple_allowed: false,
                        required: false,
                        supports_doc_comments: false,
                        setter: |command: &mut Self, _, _, _, _| {
                            command.allow_address_overlap = true;
                            false
                        },
                    },
                ),
                (
                    "repeat",
                    PropertyInfo {
                        allowed_expression_types: vec![Expression::Repeat(Default::default())],
                        multiple_allowed: false,
                        required: false,
                        supports_doc_comments: false,
                        setter: |command: &mut Self, property, _, _, _| {
                            let repeat = property.expression.as_repeat().unwrap();
                            command.repeat = Some(Repeat {
                                source: match repeat.source {
                                    device_driver_parser::RepeatSource::Count(count) => {
                                        RepeatSource::Count(count as u64)
                                    }
                                    device_driver_parser::RepeatSource::Enum(ident) => {
                                        RepeatSource::Enum(
                                            IdentifierRef::new(ident.val.into())
                                                .with_span(ident.span),
                                        )
                                    }
                                },
                                stride: repeat.stride as i128,
                            });
                            false
                        },
                    },
                ),
                (
                    "fields-in",
                    PropertyInfo {
                        allowed_expression_types: vec![
                            Expression::TypeReference(device_driver_parser::Ident {
                                val: "MyFieldset",
                                span: Span::default(),
                            }),
                            Expression::SubNode(Box::new(Node {
                                doc_comments: vec![],
                                node_type: device_driver_parser::Ident {
                                    val: "fieldset",
                                    span: Default::default(),
                                },
                                name: device_driver_parser::Ident {
                                    val: "MyFieldSet",
                                    span: Default::default(),
                                },
                                type_specifier: None,
                                properties: vec![],
                                short_properties: vec![],
                                sub_nodes: vec![],
                                span: Default::default(),
                            })),
                        ],
                        multiple_allowed: false,
                        required: false,
                        supports_doc_comments: false,
                        setter: |command: &mut Self,
                                 property,
                                 node,
                                 diagnostics,
                                 sibling_objects| {
                            match &property.expression.value {
                                Expression::TypeReference(ident) => {
                                    command.field_set_ref_in =
                                        Some(IdentifierRef::new(ident.val.into()));
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
                                            command.field_set_ref_in = Some(fs.name().take_ref());
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
                ),
                (
                    "fields-out",
                    PropertyInfo {
                        allowed_expression_types: vec![
                            Expression::TypeReference(device_driver_parser::Ident {
                                val: "MyFieldset",
                                span: Span::default(),
                            }),
                            Expression::SubNode(Box::new(Node {
                                doc_comments: vec![],
                                node_type: device_driver_parser::Ident {
                                    val: "fieldset",
                                    span: Default::default(),
                                },
                                name: device_driver_parser::Ident {
                                    val: "MyFieldSet",
                                    span: Default::default(),
                                },
                                type_specifier: None,
                                properties: vec![],
                                short_properties: vec![],
                                sub_nodes: vec![],
                                span: Default::default(),
                            })),
                        ],
                        multiple_allowed: false,
                        required: false,
                        supports_doc_comments: false,
                        setter: |command: &mut Self,
                                 property,
                                 node,
                                 diagnostics,
                                 sibling_objects| {
                            match &property.expression.value {
                                Expression::TypeReference(ident) => {
                                    command.field_set_ref_out =
                                        Some(IdentifierRef::new(ident.val.into()));
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
                                            command.field_set_ref_out = Some(fs.name().take_ref());
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
                ),
            ]
            .into()
        })
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

    // TODO: This needs to become a new `supported_short_properties`
    fn supported_properties() -> &'static Properties<Self> {
        static MAP: OnceLock<Properties<Field>> = OnceLock::new();
        MAP.get_or_init(|| {
            [(
                "address",
                PropertyInfo {
                    allowed_expression_types: vec![
                        Expression::Number(0),
                        Expression::AddressRange { end: 8, start: 0 },
                    ],
                    multiple_allowed: false,
                    required: true,
                    supports_doc_comments: false,
                    setter: |field: &mut Self, property, _, diagnostics, _| {
                        let u32_range = 0..=u32::MAX as i128;

                        field.field_address = match property.expression.value {
                            Expression::AddressRange { end, start }
                                if u32_range.contains(&end) && u32_range.contains(&start) =>
                            {
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
            )]
            .into()
        })
    }

    fn base_type(&mut self) -> Option<&mut Spanned<BaseType>> {
        Some(&mut self.base_type)
    }
}
