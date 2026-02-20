use std::{
    collections::HashMap,
    mem::{self, Discriminant},
    str::FromStr,
};

use crate::model::{Device, DeviceConfig, Manifest, Object};
use device_driver_common::{
    identifier::Identifier,
    span::{SpanExt, Spanned},
    specifiers::{ByteOrder, NodeType},
};
use device_driver_diagnostics::{
    Diagnostics,
    errors::{
        DocCommentsNotSupported, DuplicateProperty, InvalidExpressionType, InvalidIdentifier,
        InvalidNodeType, InvalidPropertyName, NameNotSupported, NameRequired, UnknownNodeType,
    },
};
use device_driver_parser::{Ast, Expression, Node};
use itertools::Itertools;

pub fn lower(ast: Ast, diagnostics: &mut Diagnostics) -> Manifest {
    let mut manifest = Manifest {
        objects: Vec::new(),
        config: DeviceConfig::default(),
    };

    // println!("{ast:#?}");

    for node in ast.nodes {
        lower_node(
            &node,
            Some(&mut manifest.config),
            &mut manifest.objects,
            None,
            &[
                NodeType::Global,
                NodeType::Device,
                NodeType::FieldSet,
                NodeType::Enum,
                NodeType::Extern,
            ],
            diagnostics,
        );
    }

    manifest
}

fn lower_node(
    node: &Node,
    device_config: Option<&mut DeviceConfig>,
    objects: &mut Vec<Object>,
    parent_node_type: Option<Spanned<NodeType>>,
    allowed_node_types: &[NodeType],
    diagnostics: &mut Diagnostics,
) {
    let Ok(node_type) = NodeType::from_str(node.node_type.val) else {
        diagnostics.add(UnknownNodeType {
            node_type: node.node_type.span,
        });
        return;
    };
    let node_type = node_type.with_span(node.node_type.span);

    if !allowed_node_types.contains(&node_type) {
        diagnostics.add(InvalidNodeType {
            node_type: node_type.span,
            parent_node_type,
            allowed_node_types: allowed_node_types.to_vec(),
        });
        return;
    }

    match node_type.value {
        NodeType::Global => {
            let mut global_config = DeviceConfig::default();
            if node_parser(&mut global_config, node_type, node, diagnostics).is_ok() {
                *device_config.unwrap() = global_config;
            }
        }
        NodeType::Device => {
            let mut device = Device::default();
            if node_parser(&mut device, node_type, node, diagnostics).is_ok() {
                objects.push(Object::Device(device));
            }
        }
        NodeType::Block => todo!(),
        NodeType::Register => todo!(),
        NodeType::Command => todo!(),
        NodeType::Buffer => todo!(),
        NodeType::FieldSet => todo!(),
        NodeType::Enum => todo!(),
        NodeType::Extern => todo!(),
    }
}

fn node_parser(
    target: &mut impl Shape,
    node_type: Spanned<NodeType>,
    node: &Node,
    diagnostics: &mut Diagnostics,
) -> Result<(), ()> {
    // Doc comments

    match target.doc_comments() {
        Some(doc_comments) => {
            *doc_comments = node.doc_comments.iter().map(|c| c.value).join("\n");
        }
        None => {
            if !node.doc_comments.is_empty() {
                diagnostics.add(DocCommentsNotSupported {
                    doc_comments: node.doc_comments.iter().map(|c| c.span).collect(),
                    node_type,
                });
            }
        }
    }

    // Object name

    match (target.name(), &node.name) {
        (Some(target_name), Some(node_name)) => match Identifier::try_parse(node_name.val) {
            Ok(ident) => *target_name = ident.with_span(node_name.span),
            Err(e) => {
                diagnostics.add(InvalidIdentifier::new(e, node_name.span));
                // Can't continue with this node when there's no name
                return Err(());
            }
        },
        (Some(_), None) => {
            diagnostics.add(NameRequired { node_type });
            // Can't continue with this node when there's no name
            return Err(());
        }
        (None, Some(node_name)) => {
            diagnostics.add(NameNotSupported {
                name: node_name.span,
                node_type,
            });
        }
        (None, None) => {}
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
                            node_type,
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

        // Get the current expression and transmute it to the static lifetime.
        // This is explicitly allowed in the std docs: https://doc.rust-lang.org/std/mem/fn.discriminant.html
        let current_expression_type = unsafe {
            mem::transmute::<Discriminant<Expression<'_>>, Discriminant<Expression<'static>>>(
                mem::discriminant(&property.expression.value),
            )
        };
        let expression_supported = property_support
            .allowed_expression_types
            .iter()
            .any(|allowed_expression_type| &current_expression_type == allowed_expression_type);

        if !expression_supported {
            diagnostics.add(InvalidExpressionType {
                expression: property.expression.span,
                node_type,
            });
            continue;
        }

        (property_support.setter)(target, &property.expression);

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

    // Sub nodes

    if let Some((objects, allowed_node_types)) = target.child_objects() {
        for sub_node in node.sub_nodes.iter() {
            lower_node(
                sub_node,
                None,
                objects,
                Some(node_type),
                &allowed_node_types,
                diagnostics,
            );
        }
    } else if !node.sub_nodes.is_empty() {
        todo!("diagnostic: node type doesn't support sub-nodes");
    }

    Ok(())
}

trait Shape {
    fn doc_comments(&mut self) -> Option<&mut String> {
        None
    }

    fn name(&mut self) -> Option<&mut Spanned<Identifier>> {
        None
    }

    /// All the supported properties. An empty name string matches anything, None only matches anonymous properties
    fn supported_properties(&mut self) -> HashMap<Option<&'static str>, PropertySupport<Self>> {
        HashMap::new()
    }

    /// Returns Some if the shape support child objects. It will be populated from the sub-nodes.
    /// The vec are the objects, the slice is the allowed node types.
    fn child_objects(&mut self) -> Option<(&mut Vec<Object>, Vec<NodeType>)> {
        None
    }
}

struct PropertySupport<T: ?Sized> {
    allowed_expression_types: Vec<Discriminant<Expression<'static>>>,
    /// If true, multiple of these properties are allowed
    multiple_allowed: bool,
    setter: fn(&mut T, &Spanned<Expression<'_>>),
}

impl Shape for DeviceConfig {
    fn supported_properties(&mut self) -> HashMap<Option<&'static str>, PropertySupport<Self>> {
        [
            (
                Some("register-access"),
                PropertySupport {
                    allowed_expression_types: vec![mem::discriminant(&Expression::Access(
                        device_driver_common::specifiers::Access::RO,
                    ))],
                    multiple_allowed: false,
                    setter: |dc: &mut Self, val| {
                        dc.register_access = Some(val.as_access().unwrap())
                    },
                },
            ),
            (
                Some("field-access"),
                PropertySupport {
                    allowed_expression_types: vec![mem::discriminant(&Expression::Access(
                        device_driver_common::specifiers::Access::RO,
                    ))],
                    multiple_allowed: false,
                    setter: |dc: &mut Self, val| dc.field_access = Some(val.as_access().unwrap()),
                },
            ),
            (
                Some("buffer-access"),
                PropertySupport {
                    allowed_expression_types: vec![mem::discriminant(&Expression::Access(
                        device_driver_common::specifiers::Access::RO,
                    ))],
                    multiple_allowed: false,
                    setter: |dc: &mut Self, val| dc.buffer_access = Some(val.as_access().unwrap()),
                },
            ),
            (
                Some("byte-order"),
                PropertySupport {
                    allowed_expression_types: vec![mem::discriminant(&Expression::ByteOrder(
                        ByteOrder::BE,
                    ))],
                    multiple_allowed: false,
                    setter: |dc: &mut Self, val| dc.byte_order = Some(val.as_byte_order().unwrap()),
                },
            ),
            (
                Some("register-address-type"),
                PropertySupport {
                    allowed_expression_types: vec![mem::discriminant(&Expression::Integer(
                        device_driver_common::specifiers::Integer::I16,
                    ))],
                    multiple_allowed: false,
                    setter: |dc: &mut Self, val| {
                        dc.register_address_type =
                            Some(val.as_integer().unwrap().with_span(val.span))
                    },
                },
            ),
            (
                Some("command-address-type"),
                PropertySupport {
                    allowed_expression_types: vec![mem::discriminant(&Expression::Integer(
                        device_driver_common::specifiers::Integer::I16,
                    ))],
                    multiple_allowed: false,
                    setter: |dc: &mut Self, val| {
                        dc.command_address_type =
                            Some(val.as_integer().unwrap().with_span(val.span))
                    },
                },
            ),
            (
                Some("buffer-address-type"),
                PropertySupport {
                    allowed_expression_types: vec![mem::discriminant(&Expression::Integer(
                        device_driver_common::specifiers::Integer::I16,
                    ))],
                    multiple_allowed: false,
                    setter: |dc: &mut Self, val| {
                        dc.buffer_address_type = Some(val.as_integer().unwrap().with_span(val.span))
                    },
                },
            ),
            // TODO: name-word-boundaries
            // TODO: defmt-feature
        ]
        .into()
    }
}

impl Shape for Device {
    fn doc_comments(&mut self) -> Option<&mut String> {
        Some(&mut self.description)
    }

    fn name(&mut self) -> Option<&mut Spanned<Identifier>> {
        Some(&mut self.name)
    }

    fn supported_properties(&mut self) -> HashMap<Option<&'static str>, PropertySupport<Self>> {
        [
            (
                Some("register-access"),
                PropertySupport {
                    allowed_expression_types: vec![mem::discriminant(&Expression::Access(
                        device_driver_common::specifiers::Access::RO,
                    ))],
                    multiple_allowed: false,
                    setter: |dev: &mut Self, val| {
                        dev.device_config.register_access = Some(val.as_access().unwrap())
                    },
                },
            ),
            (
                Some("field-access"),
                PropertySupport {
                    allowed_expression_types: vec![mem::discriminant(&Expression::Access(
                        device_driver_common::specifiers::Access::RO,
                    ))],
                    multiple_allowed: false,
                    setter: |dev: &mut Self, val| {
                        dev.device_config.field_access = Some(val.as_access().unwrap())
                    },
                },
            ),
            (
                Some("buffer-access"),
                PropertySupport {
                    allowed_expression_types: vec![mem::discriminant(&Expression::Access(
                        device_driver_common::specifiers::Access::RO,
                    ))],
                    multiple_allowed: false,
                    setter: |dev: &mut Self, val| {
                        dev.device_config.buffer_access = Some(val.as_access().unwrap())
                    },
                },
            ),
            (
                Some("byte-order"),
                PropertySupport {
                    allowed_expression_types: vec![mem::discriminant(&Expression::ByteOrder(
                        ByteOrder::BE,
                    ))],
                    multiple_allowed: false,
                    setter: |dev: &mut Self, val| {
                        dev.device_config.byte_order = Some(val.as_byte_order().unwrap())
                    },
                },
            ),
            (
                Some("register-address-type"),
                PropertySupport {
                    allowed_expression_types: vec![mem::discriminant(&Expression::Integer(
                        device_driver_common::specifiers::Integer::I16,
                    ))],
                    multiple_allowed: false,
                    setter: |dev: &mut Self, val| {
                        dev.device_config.register_address_type =
                            Some(val.as_integer().unwrap().with_span(val.span))
                    },
                },
            ),
            (
                Some("command-address-type"),
                PropertySupport {
                    allowed_expression_types: vec![mem::discriminant(&Expression::Integer(
                        device_driver_common::specifiers::Integer::I16,
                    ))],
                    multiple_allowed: false,
                    setter: |dev: &mut Self, val| {
                        dev.device_config.command_address_type =
                            Some(val.as_integer().unwrap().with_span(val.span))
                    },
                },
            ),
            (
                Some("buffer-address-type"),
                PropertySupport {
                    allowed_expression_types: vec![mem::discriminant(&Expression::Integer(
                        device_driver_common::specifiers::Integer::I16,
                    ))],
                    multiple_allowed: false,
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
