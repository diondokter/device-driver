use std::{
    borrow::Cow,
    collections::HashMap,
    mem::{self, Discriminant, discriminant},
    str::FromStr,
};

use crate::model::{Manifest, Object};
use device_driver_common::{
    identifier::{Identifier, IdentifierRef, IdentifierType, Type},
    span::{Span, SpanExt, Spanned},
    specifiers::{BaseType, NodeType, Repeat, RepeatSource, TypeConversion},
};
use device_driver_diagnostics::{
    Diagnostics,
    errors::{
        DuplicateProperty, IgnoredDocCommentOnProperty, InvalidAutoIdentifier,
        InvalidExpressionType, InvalidIdentifier, InvalidNodeType, InvalidPropertyName,
        InvalidRepeat, InvalidShortProperty, InvalidSubnode, InvalidTypeConversion,
        InvalidTypeSpecifier, MissingRequiredProperty, UnknownNodeType,
    },
};
use device_driver_parser::{Ast, Expression, Ident, Node, Property};
use itertools::Itertools;

#[cfg(feature = "gen-docs")]
pub mod gen_docs;
mod shape_impls;

pub fn lower(ast: Ast, diagnostics: &mut Diagnostics) -> Manifest {
    let Some(root_node) = ast.root_node else {
        return Default::default();
    };

    let result = lower_node(
        &root_node,
        None,
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
    parent_node_name: Option<Ident>,
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
        NodeType::Manifest => match parse_node_to_shape(node, parent_node_name, diagnostics) {
            Ok((val, siblings)) => {
                assert!(siblings.is_empty(), "Manifest has no siblings");
                LowerResult::Manifest(val)
            }
            Err(siblings) => LowerResult::Error(siblings),
        },
        NodeType::Device => match parse_node_to_shape(node, parent_node_name, diagnostics) {
            Ok((val, siblings)) => LowerResult::Objects(Object::Device(val), siblings),
            Err(siblings) => LowerResult::Error(siblings),
        },
        NodeType::Block => match parse_node_to_shape(node, parent_node_name, diagnostics) {
            Ok((val, siblings)) => LowerResult::Objects(Object::Block(val), siblings),
            Err(siblings) => LowerResult::Error(siblings),
        },
        NodeType::Register => match parse_node_to_shape(node, parent_node_name, diagnostics) {
            Ok((val, siblings)) => LowerResult::Objects(Object::Register(val), siblings),
            Err(siblings) => LowerResult::Error(siblings),
        },
        NodeType::Command => match parse_node_to_shape(node, parent_node_name, diagnostics) {
            Ok((val, siblings)) => LowerResult::Objects(Object::Command(val), siblings),
            Err(siblings) => LowerResult::Error(siblings),
        },
        NodeType::Buffer => match parse_node_to_shape(node, parent_node_name, diagnostics) {
            Ok((val, siblings)) => LowerResult::Objects(Object::Buffer(val), siblings),
            Err(siblings) => LowerResult::Error(siblings),
        },
        NodeType::FieldSet => match parse_node_to_shape(node, parent_node_name, diagnostics) {
            Ok((val, siblings)) => LowerResult::Objects(Object::FieldSet(val), siblings),
            Err(siblings) => LowerResult::Error(siblings),
        },
        NodeType::Enum => match parse_node_to_shape(node, parent_node_name, diagnostics) {
            Ok((val, siblings)) => LowerResult::Objects(Object::Enum(val), siblings),
            Err(siblings) => LowerResult::Error(siblings),
        },
        NodeType::Extern => match parse_node_to_shape(node, parent_node_name, diagnostics) {
            Ok((val, siblings)) => LowerResult::Objects(Object::Extern(val), siblings),
            Err(siblings) => LowerResult::Error(siblings),
        },
        NodeType::Field => match parse_node_to_shape(node, parent_node_name, diagnostics) {
            Ok((val, siblings)) => LowerResult::Objects(Object::Field(val), siblings),
            Err(siblings) => LowerResult::Error(siblings),
        },
    }
}

fn parse_node_to_shape<'src, S: Shape>(
    node: &Node<'src>,
    parent_node_name: Option<Ident<'src>>,
    diagnostics: &mut Diagnostics,
) -> Result<(S, Vec<Object>), Vec<Object>> {
    let mut target = S::default();
    let mut sibling_objects = Vec::new();
    let mut error = false;

    *target.span() = node.span;

    // Doc comments

    *target.doc_comments() = node.doc_comments.iter().map(|c| c.value).join("\n");

    // Object name

    match (node.name.is_auto(), parent_node_name) {
        (true, Some(parent_node_name)) => {
            match Identifier::try_parse(parent_node_name.val) {
                Ok(ident) => *target.name() = ident.with_span(node.name.span),
                Err(_e) => {
                    // We don't need to emit a diagnostic since the parent will already have a diagnostic.
                    // Can't continue with this node when there's no name
                    error = true;
                }
            }
        }
        (true, None) => {
            diagnostics.add(InvalidAutoIdentifier {
                auto_identifier: node.name.span,
            });
            // Can't continue with this node when there's no name
            error = true;
        }
        (false, _) => {
            match Identifier::try_parse(node.name.val) {
                Ok(ident) => *target.name() = ident.with_span(node.name.span),
                Err(e) => {
                    diagnostics.add(InvalidIdentifier::new(e, node.name.span));
                    // Can't continue with this node when there's no name
                    error = true;
                }
            }
        }
    }

    // Repeat

    match (target.repeat(), node.repeat) {
        (None, Some(node_repeat)) => {
            diagnostics.add(InvalidRepeat {
                repeat: node_repeat.span,
                node_type: S::NODE_TYPE.with_span(node.node_type.span),
            });
        }
        (Some(target_repeat), Some(node_repeat)) => {
            *target_repeat = Some(Repeat {
                source: match node_repeat.source {
                    device_driver_parser::RepeatSource::Count(count) => RepeatSource::Count(count),
                    device_driver_parser::RepeatSource::Enum(ident) => RepeatSource::Enum(
                        IdentifierRef::new(ident.val.into()).with_span(ident.span),
                    ),
                },
                stride: (node_repeat.stride.value as i128).with_span(node_repeat.stride.span),
            })
        }
        (_, None) => {}
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
                        Some(IdentifierRef::<Type>::new(ident.val.into()).with_span(ident.span))
                    }
                    device_driver_parser::TypeConversion::Subnode(sub_node) => {
                        let sub_node = lower_node(
                            sub_node,
                            Some(NodeType::Field.with_span(node.node_type.span)),
                            Some(node.name),
                            &[NodeType::Enum, NodeType::Extern],
                            diagnostics,
                        );

                        match sub_node {
                            LowerResult::Manifest(_) => unreachable!(),
                            LowerResult::Objects(object, objects) => {
                                let reference = object
                                    .name()
                                    .clone()
                                    // The only allowed subnodes are types, so this should be fine
                                    .cast_assert()
                                    .take_ref()
                                    .with_span(object.name_span());
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

        error |= (property_info.setter)(SetterArgs {
            target_object: &mut target,
            property,
            node,
            diagnostics,
            sibling_objects: &mut sibling_objects,
        });

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
                    got: short_property.to_string(),
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

        error |= (property_info.setter)(SetterArgs {
            target_object: &mut target,
            property: &Property {
                doc_comments: Vec::new(),
                name: Ident::new("", short_property.span),
                expression: short_property.clone(),
            }
            .with_span(short_property.span),
            node,
            diagnostics,
            sibling_objects: &mut sibling_objects,
        });

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
                None,
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
    type NameIdentifierType: IdentifierType + Default;

    fn doc_comments(&mut self) -> &mut String;
    fn name(&mut self) -> &mut Spanned<Identifier<Self::NameIdentifierType>>;

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

    fn repeat(&mut self) -> Option<&mut Option<Repeat>> {
        None
    }

    fn span(&mut self) -> &mut Span;
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
    /// Doesn't work well with [`Self::multiple_allowed`] set at the same time.
    required: bool,
    /// If false, a warning is emitted when the property has doc comments
    supports_doc_comments: bool,
    /// If setter returns true, there's an error
    setter: for<'a, 'src> fn(SetterArgs<'a, 'src, T>) -> bool,
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

struct SetterArgs<'a, 'src, T: ?Sized> {
    /// The target object that needs a property set
    target_object: &'a mut T,
    /// The property that needs to be set
    property: &'a Spanned<Property<'src>>,
    /// The node that's being parsed
    node: &'a Node<'src>,
    diagnostics: &'a mut Diagnostics,
    sibling_objects: &'a mut Vec<Object>,
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
