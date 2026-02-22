use std::{collections::HashMap, mem, str::FromStr};

use crate::model::{Block, Device, Manifest, Object, Register};
use device_driver_common::{
    identifier::{Identifier, IdentifierRef},
    span::{Span, SpanExt, Spanned},
    specifiers::{NodeType, Repeat, RepeatSource, ResetValue},
};
use device_driver_diagnostics::{
    Diagnostics,
    errors::{
        DuplicateProperty, InvalidExpressionType, InvalidIdentifier, InvalidNodeType,
        InvalidPropertyName, MissingRequiredProperty, UnknownNodeType,
    },
};
use device_driver_parser::{Ast, Expression, Node};
use itertools::Itertools;

pub fn lower(ast: Ast, diagnostics: &mut Diagnostics) -> Manifest {
    println!("{ast:#?}");

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
        Some(LowerResult::Manifest(m)) => m,
        Some(LowerResult::Object(Object::Device(d))) => d.into(),
        Some(_) => unreachable!(),
        None => Default::default(),
    }
}

enum LowerResult {
    Manifest(Manifest),
    Object(Object),
}

impl LowerResult {
    fn into_object(self) -> Option<Object> {
        if let Self::Object(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

fn lower_node(
    node: &Node,
    parent_node_type: Option<Spanned<NodeType>>,
    allowed_node_types: &[NodeType],
    diagnostics: &mut Diagnostics,
) -> Option<LowerResult> {
    let Ok(node_type) = NodeType::from_str(node.node_type.val) else {
        diagnostics.add(UnknownNodeType {
            node_type: node.node_type.span,
            allowed_node_types: allowed_node_types.to_vec(),
        });
        return None;
    };
    let node_type = node_type.with_span(node.node_type.span);

    if !allowed_node_types.contains(&node_type) {
        diagnostics.add(InvalidNodeType {
            node_type: node_type.span,
            parent_node_type,
            allowed_node_types: allowed_node_types.to_vec(),
        });
        return None;
    }

    match node_type.value {
        NodeType::Manifest => {
            if let Ok(manifest) = node_parser(node, diagnostics) {
                return Some(LowerResult::Manifest(manifest));
            }
        }
        NodeType::Device => {
            if let Ok(device) = node_parser(node, diagnostics) {
                return Some(LowerResult::Object(Object::Device(device)));
            }
        }
        NodeType::Block => {
            if let Ok(block) = node_parser(node, diagnostics) {
                return Some(LowerResult::Object(Object::Block(block)));
            }
        }
        NodeType::Register => {
            if let Ok(register) = node_parser(node, diagnostics) {
                return Some(LowerResult::Object(Object::Register(register)));
            }
        }
        NodeType::Command => todo!(),
        NodeType::Buffer => todo!(),
        NodeType::FieldSet => todo!(),
        NodeType::Enum => todo!(),
        NodeType::Extern => todo!(),
    }

    None
}

fn node_parser<S: Shape>(node: &Node, diagnostics: &mut Diagnostics) -> Result<S, ()> {
    let mut target = S::default();
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

    // Base type: TODO

    // Conversion: TODO

    // Properties

    let mut possible_properties = target.supported_properties();
    let mut removed_possible_properties = HashMap::new();
    for property in &node.properties {
        let property_name = &property.name.as_ref().map(|n| n.val);
        let property_fallback_name = if property.name.is_some() {
            &Some("")
        } else {
            &None
        };

        let Some(property_support) = possible_properties
            .get(property_name)
            .or_else(|| possible_properties.get(property_fallback_name))
        else {
            match &property.name {
                Some(name) => {
                    if let Some(original) = removed_possible_properties.get(property_name).copied()
                    {
                        diagnostics.add(DuplicateProperty {
                            original,
                            duplicate: name.span,
                        });
                    } else {
                        diagnostics.add(InvalidPropertyName {
                            property: name.span,
                            node_type: S::NODE_TYPE.with_span(node.node_type.span),
                            expected_names: target
                                .supported_properties()
                                .keys()
                                .filter_map(|k| *k)
                                .collect(),
                        });
                    }
                }
                None => {
                    // Anonymous properties not allowed or expression type already seen and removed
                    todo!()
                }
            }
            continue;
        };

        let current_expression_type = mem::discriminant(&property.expression.value);

        let expression_supported =
            property_support
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
                valid_expression_types: property_support
                    .allowed_expression_types
                    .iter()
                    .map(|e| e.to_string())
                    .collect(),
                valid_expression_values: property_support
                    .allowed_expression_types
                    .iter()
                    .map(|e| e.get_human_string())
                    .collect(),
            });
            continue;
        }

        (property_support.setter)(&mut target, &property.expression);

        if !property_support.multiple_allowed {
            possible_properties
                .remove(property_name)
                .or_else(|| possible_properties.remove(property_fallback_name));

            removed_possible_properties.insert(
                *property_name,
                property
                    .name
                    .as_ref()
                    .map(|n| n.span)
                    .unwrap_or(property.span),
            );
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
                required_property_name: missing_name.map(|s| s.to_string()),
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

    if let Some((objects, allowed_node_types)) = target.child_objects() {
        for sub_node in node.sub_nodes.iter() {
            if let Some(result) = lower_node(
                sub_node,
                Some(S::NODE_TYPE.with_span(node.node_type.span)),
                &allowed_node_types,
                diagnostics,
            ) {
                objects.push(
                    result
                        .into_object()
                        .expect("Always yields an object this deep down in parsing"),
                );
            }
        }
    } else if !node.sub_nodes.is_empty() {
        todo!("diagnostic: node type doesn't support sub-nodes");
    }

    if !error { Ok(target) } else { Err(()) }
}

trait Shape: Default {
    const NODE_TYPE: NodeType;

    fn doc_comments(&mut self) -> &mut String;
    fn name(&mut self) -> &mut Spanned<Identifier>;

    /// All the supported properties. An empty name string matches anything, None only matches anonymous properties
    fn supported_properties(&mut self) -> HashMap<Option<&'static str>, PropertyInfo<Self>> {
        HashMap::new()
    }

    /// Returns Some if the shape support child objects. It will be populated from the sub-nodes.
    /// The vec are the objects, the slice is the allowed node types.
    fn child_objects(&mut self) -> Option<(&mut Vec<Object>, Vec<NodeType>)> {
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
    setter: fn(&mut T, &Spanned<Expression<'_>>),
}

impl Shape for Manifest {
    const NODE_TYPE: NodeType = NodeType::Manifest;

    fn doc_comments(&mut self) -> &mut String {
        &mut self.description
    }

    fn name(&mut self) -> &mut Spanned<Identifier> {
        &mut self.name
    }

    fn supported_properties(&mut self) -> HashMap<Option<&'static str>, PropertyInfo<Self>> {
        [
            (
                Some("register-access"),
                PropertyInfo {
                    allowed_expression_types: vec![Expression::Access(Default::default())],
                    multiple_allowed: false,
                    required: false,
                    setter: |manifest: &mut Self, val| {
                        manifest.config.register_access = Some(val.as_access().unwrap())
                    },
                },
            ),
            (
                Some("field-access"),
                PropertyInfo {
                    allowed_expression_types: vec![Expression::Access(Default::default())],
                    multiple_allowed: false,
                    required: false,
                    setter: |manifest: &mut Self, val| {
                        manifest.config.field_access = Some(val.as_access().unwrap())
                    },
                },
            ),
            (
                Some("buffer-access"),
                PropertyInfo {
                    allowed_expression_types: vec![Expression::Access(Default::default())],
                    multiple_allowed: false,
                    required: false,
                    setter: |manifest: &mut Self, val| {
                        manifest.config.buffer_access = Some(val.as_access().unwrap())
                    },
                },
            ),
            (
                Some("byte-order"),
                PropertyInfo {
                    allowed_expression_types: vec![Expression::ByteOrder(Default::default())],
                    multiple_allowed: false,
                    required: false,
                    setter: |manifest: &mut Self, val| {
                        manifest.config.byte_order = Some(val.as_byte_order().unwrap())
                    },
                },
            ),
            (
                Some("register-address-type"),
                PropertyInfo {
                    allowed_expression_types: vec![Expression::Integer(Default::default())],
                    multiple_allowed: false,
                    required: false,
                    setter: |manifest: &mut Self, val| {
                        manifest.config.register_address_type =
                            Some(val.as_integer().unwrap().with_span(val.span))
                    },
                },
            ),
            (
                Some("command-address-type"),
                PropertyInfo {
                    allowed_expression_types: vec![Expression::Integer(Default::default())],
                    multiple_allowed: false,
                    required: false,
                    setter: |manifest: &mut Self, val| {
                        manifest.config.command_address_type =
                            Some(val.as_integer().unwrap().with_span(val.span))
                    },
                },
            ),
            (
                Some("buffer-address-type"),
                PropertyInfo {
                    allowed_expression_types: vec![Expression::Integer(Default::default())],
                    multiple_allowed: false,
                    required: false,
                    setter: |manifest: &mut Self, val| {
                        manifest.config.buffer_address_type =
                            Some(val.as_integer().unwrap().with_span(val.span))
                    },
                },
            ),
            // TODO: name-word-boundaries
            // TODO: defmt-feature
        ]
        .into()
    }

    fn child_objects(&mut self) -> Option<(&mut Vec<Object>, Vec<NodeType>)> {
        Some((
            &mut self.objects,
            vec![
                NodeType::Device,
                NodeType::FieldSet,
                NodeType::Enum,
                NodeType::Extern,
            ],
        ))
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

    fn supported_properties(&mut self) -> HashMap<Option<&'static str>, PropertyInfo<Self>> {
        [
            (
                Some("register-access"),
                PropertyInfo {
                    allowed_expression_types: vec![Expression::Access(Default::default())],
                    multiple_allowed: false,
                    required: false,
                    setter: |dev: &mut Self, val| {
                        dev.device_config.register_access = Some(val.as_access().unwrap())
                    },
                },
            ),
            (
                Some("field-access"),
                PropertyInfo {
                    allowed_expression_types: vec![Expression::Access(Default::default())],
                    multiple_allowed: false,
                    required: false,
                    setter: |dev: &mut Self, val| {
                        dev.device_config.field_access = Some(val.as_access().unwrap())
                    },
                },
            ),
            (
                Some("buffer-access"),
                PropertyInfo {
                    allowed_expression_types: vec![Expression::Access(Default::default())],
                    multiple_allowed: false,
                    required: false,
                    setter: |dev: &mut Self, val| {
                        dev.device_config.buffer_access = Some(val.as_access().unwrap())
                    },
                },
            ),
            (
                Some("byte-order"),
                PropertyInfo {
                    allowed_expression_types: vec![Expression::ByteOrder(Default::default())],
                    multiple_allowed: false,
                    required: false,
                    setter: |dev: &mut Self, val| {
                        dev.device_config.byte_order = Some(val.as_byte_order().unwrap())
                    },
                },
            ),
            (
                Some("register-address-type"),
                PropertyInfo {
                    allowed_expression_types: vec![Expression::Integer(Default::default())],
                    multiple_allowed: false,
                    required: false,
                    setter: |dev: &mut Self, val| {
                        dev.device_config.register_address_type =
                            Some(val.as_integer().unwrap().with_span(val.span))
                    },
                },
            ),
            (
                Some("command-address-type"),
                PropertyInfo {
                    allowed_expression_types: vec![Expression::Integer(Default::default())],
                    multiple_allowed: false,
                    required: false,
                    setter: |dev: &mut Self, val| {
                        dev.device_config.command_address_type =
                            Some(val.as_integer().unwrap().with_span(val.span))
                    },
                },
            ),
            (
                Some("buffer-address-type"),
                PropertyInfo {
                    allowed_expression_types: vec![Expression::Integer(Default::default())],
                    multiple_allowed: false,
                    required: false,
                    setter: |dev: &mut Self, val| {
                        dev.device_config.buffer_address_type =
                            Some(val.as_integer().unwrap().with_span(val.span))
                    },
                },
            ),
            // TODO: name-word-boundaries
            // TODO: defmt-feature
        ]
        .into()
    }

    fn child_objects(&mut self) -> Option<(&mut Vec<Object>, Vec<NodeType>)> {
        Some((
            &mut self.objects,
            vec![
                NodeType::Block,
                NodeType::Register,
                NodeType::Command,
                NodeType::Buffer,
                NodeType::FieldSet,
                NodeType::Enum,
                NodeType::Extern,
            ],
        ))
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

    fn supported_properties(&mut self) -> HashMap<Option<&'static str>, PropertyInfo<Self>> {
        [
            (
                Some("address-offset"),
                PropertyInfo {
                    allowed_expression_types: vec![Expression::Number(Default::default())],
                    multiple_allowed: false,
                    required: true,
                    setter: |block: &mut Self, expression| {
                        block.address_offset =
                            expression.as_number().unwrap().with_span(expression.span)
                    },
                },
            ),
            (
                Some("repeat"),
                PropertyInfo {
                    allowed_expression_types: vec![Expression::Repeat(Default::default())],
                    multiple_allowed: false,
                    required: false,
                    setter: |block: &mut Self, expression| {
                        block.address_offset =
                            expression.as_number().unwrap().with_span(expression.span)
                    },
                },
            ),
        ]
        .into()
    }

    fn child_objects(&mut self) -> Option<(&mut Vec<Object>, Vec<NodeType>)> {
        Some((
            &mut self.objects,
            vec![
                NodeType::Block,
                NodeType::Register,
                NodeType::Command,
                NodeType::Buffer,
                NodeType::FieldSet,
                NodeType::Enum,
                NodeType::Extern,
            ],
        ))
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

    fn supported_properties(&mut self) -> HashMap<Option<&'static str>, PropertyInfo<Self>> {
        [
            (
                Some("address"),
                PropertyInfo {
                    allowed_expression_types: vec![Expression::Number(Default::default())],
                    multiple_allowed: false,
                    required: true,
                    setter: |r: &mut Self, e| r.address = e.as_number().unwrap().with_span(e.span),
                },
            ),
            (
                Some("access"),
                PropertyInfo {
                    allowed_expression_types: vec![Expression::Access(Default::default())],
                    multiple_allowed: false,
                    required: false,
                    setter: |r: &mut Self, e| r.access = e.as_access().unwrap(),
                },
            ),
            (
                Some("address-overlap"),
                PropertyInfo {
                    allowed_expression_types: vec![Expression::Allow],
                    multiple_allowed: false,
                    required: false,
                    setter: |r: &mut Self, _| r.allow_address_overlap = true,
                },
            ),
            (
                Some("reset"),
                PropertyInfo {
                    allowed_expression_types: vec![
                        Expression::ResetArray(vec![12, 34]),
                        Expression::ResetNumber(1234),
                    ],
                    multiple_allowed: false,
                    required: false,
                    setter: |r: &mut Self, e| match &e.value {
                        Expression::ResetNumber(num) => {
                            r.reset_value = Some(ResetValue::Integer(*num).with_span(e.span))
                        }
                        Expression::ResetArray(bytes) => {
                            r.reset_value = Some(ResetValue::Array(bytes.clone()).with_span(e.span))
                        }
                        _ => unreachable!(),
                    },
                },
            ),
            (
                Some("repeat"),
                PropertyInfo {
                    allowed_expression_types: vec![Expression::Repeat(Default::default())],
                    multiple_allowed: false,
                    required: false,
                    setter: |r: &mut Self, e| {
                        let repeat = e.as_repeat().unwrap();
                        r.repeat = Some(Repeat {
                            source: match repeat.source {
                                device_driver_parser::RepeatSource::Count(count) => {
                                    RepeatSource::Count(count as u64)
                                }
                                device_driver_parser::RepeatSource::Enum(ident) => {
                                    RepeatSource::Enum(
                                        IdentifierRef::new(ident.val.into()).with_span(ident.span),
                                    )
                                }
                            },
                            stride: repeat.stride as i128,
                        });
                    },
                },
            ),
            (
                Some("fields"),
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
                            sub_nodes: vec![],
                            span: Default::default(),
                        })),
                    ],
                    multiple_allowed: false,
                    required: true,
                    setter: |r: &mut Self, e| match &e.value {
                        Expression::TypeReference(ident) => {
                            r.field_set_ref = IdentifierRef::new(ident.val.into())
                        }
                        Expression::SubNode(node) => todo!("Need more infra to reject nodes"),
                        _ => unreachable!(),
                    },
                },
            ),
        ]
        .into()
    }
}
