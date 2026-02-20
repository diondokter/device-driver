use std::fmt::Display;

use chumsky::{input::ValueInput, prelude::*};
use device_driver_common::{
    span::{Span, SpanExt, Spanned},
    specifiers::{Access, BaseType, ByteOrder, Integer},
};
use device_driver_diagnostics::{Diagnostics, errors::ParsingError};
use device_driver_lexer::{ParseIntRadix, ParseIntRadixError, ParseIntRadixErrorKind, Token};

pub fn parse<'src>(tokens: &[Spanned<Token<'src>>], diagnostics: &mut Diagnostics) -> Ast<'src> {
    let (ast, parse_errs) = parser()
        .map_with(|ast, e| (ast, e.span()))
        .parse(
            tokens.map(
                tokens
                    .last()
                    .map(|t| Span::from(t.span.end..t.span.end))
                    .unwrap_or_default(),
                |token| (&token.value, &token.span),
            ),
        )
        .into_output_errors();

    for error in parse_errs {
        diagnostics.add(ParsingError {
            reason: error.to_string(),
            span: *error.span(),
        });
    }

    ast.map(|(nodes, span)| Ast { nodes, span })
        .unwrap_or_default()
}

#[derive(Default, Debug)]
pub struct Ast<'src> {
    pub nodes: Vec<Node<'src>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct Node<'src> {
    pub doc_comments: Vec<Spanned<&'src str>>,
    pub node_type: Ident<'src>,
    pub name: Option<Ident<'src>>,
    pub type_specifier: Option<TypeSpecifier<'src>>,
    pub properties: Vec<Spanned<Property<'src>>>,
    pub sub_nodes: Vec<Node<'src>>,
    pub span: Span,
}

impl<'src> Display for Node<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indentation_level = f.width().unwrap_or_default();
        let indentation = format!("{:width$}", "", width = indentation_level * 4);

        for doc_comment in &self.doc_comments {
            writeln!(f, "{indentation}///{doc_comment}")?;
        }
        write!(
            f,
            "{indentation}{}{}",
            self.node_type.val,
            self.name
                .as_ref()
                .map(|ident| format!(" {}", ident.val))
                .unwrap_or_default()
        )?;

        for expression in self
            .properties
            .iter()
            .filter_map(|p| match p.is_anonymous() {
                false => None,
                true => Some(&p.value.expression),
            })
        {
            write!(f, " {expression}")?;
        }

        if let Some(type_specifier) = self.type_specifier.as_ref() {
            write!(f, " -> {}", type_specifier.base_type)?;

            if let Some(conversion) = type_specifier.conversion.as_ref() {
                write!(f, " as")?;
                if type_specifier.use_try {
                    write!(f, " try")?;
                }

                match conversion {
                    TypeConversion::Reference(ident) => write!(f, " {}", ident.val)?,
                    TypeConversion::Subnode(node) => {
                        write!(f, "\n{node:width$}", width = indentation_level + 1)?
                    }
                }
            }
        }

        if !self.sub_nodes.is_empty() || self.properties.iter().any(|p| !p.is_anonymous()) {
            writeln!(f, " {{")?;

            for (ident, expression) in self.properties.iter().filter_map(|p| match &p.name {
                Some(name) => Some((name, &p.expression.value)),
                None => None,
            }) {
                writeln!(f, "{indentation}    {}: {},", ident.val, expression)?;
            }

            for node in self.sub_nodes.iter() {
                writeln!(f, "{node:width$},", width = indentation_level + 1)?;
            }

            write!(f, "{indentation}}}")?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct TypeSpecifier<'src> {
    pub base_type: BaseType,
    pub use_try: bool,
    pub conversion: Option<TypeConversion<'src>>,
}

#[derive(Debug)]
pub enum TypeConversion<'src> {
    Reference(Ident<'src>),
    Subnode(Box<Node<'src>>),
}

#[derive(Debug)]
pub struct Property<'src> {
    pub name: Option<Ident<'src>>,
    pub expression: Spanned<Expression<'src>>,
}

impl<'src> Property<'src> {
    pub fn is_anonymous(&self) -> bool {
        self.name.is_none()
    }
}

#[derive(Debug)]
pub enum Expression<'src> {
    Range { end: i128, start: i128 },
    Repeat(Repeat<'src>),
    ResetNumber(u128),
    ResetArray(Vec<u8>),
    BaseType(BaseType),
    Integer(Integer),
    Allow,
    Number(i128),
    DefaultNumber(i128),
    CatchAllNumber(i128),
    Access(Access),
    ByteOrder(ByteOrder),
    TypeReference(Ident<'src>),
    SubNode(Box<Node<'src>>),
    Auto,
    Error,
}

impl<'src> Expression<'src> {
    pub fn as_byte_order(&self) -> Option<ByteOrder> {
        if let Self::ByteOrder(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_access(&self) -> Option<Access> {
        if let Self::Access(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_integer(&self) -> Option<Integer> {
        if let Self::Integer(v) = self {
            Some(*v)
        } else {
            None
        }
    }
}

impl<'src> Display for Expression<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Range { end, start } => write!(f, "{end}:{start}"),
            Expression::Repeat(Repeat {
                source: RepeatSource::Count(count),
                stride,
            }) => write!(f, "<{count} by {stride}>"),
            Expression::Repeat(Repeat {
                source: RepeatSource::Enum(ident),
                stride,
            }) => write!(f, "<{} by {stride}>", ident.val),
            Expression::ResetNumber(num) => write!(f, "[{num}]"),
            Expression::ResetArray(items) => write!(f, "{items:?}"),
            Expression::BaseType(base_type) => base_type.fmt(f),
            Expression::Integer(integer) => integer.fmt(f),
            Expression::Allow => write!(f, "allow"),
            Expression::Number(num) => num.fmt(f),
            Expression::DefaultNumber(num) => write!(f, "default {num}"),
            Expression::CatchAllNumber(num) => write!(f, "catch-all {num}"),
            Expression::Access(val) => val.fmt(f),
            Expression::ByteOrder(val) => val.fmt(f),
            Expression::TypeReference(ident) => ident.val.fmt(f),
            Expression::SubNode(val) => val.fmt(f),
            Expression::Auto => write!(f, "_"),
            Expression::Error => write!(f, "ERROR"),
        }
    }
}

#[derive(Debug)]
pub struct Repeat<'src> {
    pub source: RepeatSource<'src>,
    pub stride: i32,
}

#[derive(Debug)]
pub enum RepeatSource<'src> {
    Count(u32),
    Enum(Ident<'src>),
}

#[derive(Debug)]
pub struct Ident<'src> {
    pub val: &'src str,
    pub span: Span,
}

fn try_num<'tokens, 'src: 'tokens, I: ParseIntRadix>(
    num_token: Token<'src>,
    span: Span,
) -> Result<I, Rich<'tokens, Token<'src>, Span>> {
    match num_token.parse_num::<I>() {
        Ok(num) => Ok(num),
        Err(ParseIntRadixError {
            source,
            kind,
            target_bits,
            target_signed,
        }) => match kind {
            ParseIntRadixErrorKind::Overflow => Err(Rich::custom(
                span,
                format!(
                    "Number `{source}` is parsed as a {}{target_bits}, but overflows.",
                    if target_signed { 'i' } else { 'u' }
                ),
            )),
            ParseIntRadixErrorKind::Underflow => Err(Rich::custom(
                span,
                format!(
                    "Number `{source}` is parsed as a {}{target_bits}, but underflows.",
                    if target_signed { 'i' } else { 'u' }
                ),
            )),
            ParseIntRadixErrorKind::Empty => Err(Rich::custom(
                span,
                format!("Could not parse `{source}` as a number because it contains no numbers"),
            )),
        },
    }
}

pub fn parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Vec<Node<'src>>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    let node = recursive(|node| {
        let any_ident = select! {
            Token::Ident(val) = e => Ident { val, span: e.span() }
        }
        .labelled("'identifier'");

        let any_doc_comment = select! {
            Token::DocCommentLine(val) => val
        }
        .map_with(|line, extra| line.spanned(extra.span()))
        .labelled("'doc comment'");

        let any_num = select! {
            num @ Token::Num(_) => num
        }
        .labelled("'number'");

        let range = any_num
            .try_map(try_num::<i128>)
            .then_ignore(just(Token::Colon))
            .then(any_num.try_map(try_num::<i128>))
            .map(|(end, start)| Expression::Range { end, start })
            .labelled("'range'");
        let any_base_type = select! { Token::BaseType(bt) => bt }.labelled("'base type'");
        let any_integer = select! { Token::Integer(i) => i }.labelled("'integer type'");
        let repeat_expression = any_num
            .try_map(try_num::<u32>)
            .map(RepeatSource::Count)
            .or(any_ident.map(RepeatSource::Enum))
            .then(
                just(Token::By)
                    .ignore_then(any_num.try_map(try_num::<i32>))
                    .or_not(),
            )
            .map(|(source, stride)| {
                Expression::Repeat(Repeat {
                    source,
                    stride: stride.unwrap_or(1),
                })
            })
            .delimited_by(just(Token::AngleOpen), just(Token::AngleClose))
            .labelled("'<repeat>'")
            .boxed();
        let reset_expression = any_num
            .try_map(try_num::<u128>)
            .map_with(|num, extra| (num, extra.span()))
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .validate(|numbers, _, emitter| {
                if numbers.len() == 1 {
                    Expression::ResetNumber(numbers[0].0)
                } else {
                    match numbers
                        .into_iter()
                        .map(|(num, num_span)| {
                            u8::try_from(num)
                                .map_err(|_| Rich::custom(num_span, "Value must be a byte"))
                        })
                        .collect::<Result<Vec<u8>, _>>()
                    {
                        Ok(array) => Expression::ResetArray(array),
                        Err(e) => {
                            emitter.emit(e);
                            Expression::Error
                        }
                    }
                }
            })
            .delimited_by(just(Token::BracketOpen), just(Token::BracketClose))
            .labelled("'[reset]'")
            .boxed();

        // Expression without type reference
        let expression = choice((
            range,
            any_base_type.map(Expression::BaseType),
            any_integer.map(Expression::Integer),
            any_num.try_map(try_num::<i128>).map(Expression::Number),
            just(Token::Default)
                .ignore_then(any_num.try_map(try_num::<i128>))
                .map(Expression::DefaultNumber)
                .labelled("'default number'"),
            just(Token::CatchAll)
                .ignore_then(any_num.try_map(try_num::<i128>))
                .map(Expression::CatchAllNumber)
                .labelled("'catch-all number'"),
            repeat_expression,
            reset_expression,
            just(Token::Allow).map(|_| Expression::Allow),
            select! { Token::Access(val) => val }.map(Expression::Access).labelled("'access'"),
            select! { Token::ByteOrder(val) => val }.map(Expression::ByteOrder).labelled("'byte order'"),
            just(Token::Underscore).map(|_| Expression::Auto),
        ))
        .map_with(|expression, extra| expression.spanned(extra.span()))
        .boxed();

        let property = any_ident
            .then(
                just(Token::Colon).ignore_then(
                    expression.clone().or(any_ident
                        .map(Expression::TypeReference)
                        .map_with(|expression, extra| expression.spanned(extra.span()))),
                ),
            )
            .map_with(|(name, expression), extra| {
                Property {
                    name: Some(name),
                    expression,
                }
                .spanned(extra.span())
            })
            .labelled("'property'")
            .boxed();

        let type_conversion = just(Token::As).ignore_then(just(Token::Try).or_not()).then(
            node.clone()
                .map(|node| TypeConversion::Subnode(Box::new(node)))
                .or(any_ident.map(TypeConversion::Reference)),
        );
        let type_specifier = just(Token::Arrow)
            .ignore_then(any_base_type)
            .then(type_conversion.or_not())
            .map(|(base_type, conversion)| TypeSpecifier {
                base_type,
                use_try: conversion
                    .as_ref()
                    .map(|(try_token, _)| try_token.is_some())
                    .unwrap_or_default(),
                conversion: conversion.map(|(_, conversion)| conversion),
            })
            .boxed();

        let node_body = just(Token::CurlyOpen)
            .ignore_then(
                property
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .collect::<Vec<_>>(),
            )
            .then(
                node.separated_by(just(Token::Comma))
                    .allow_trailing()
                    .collect::<Vec<_>>(),
            )
            .then_ignore(just(Token::CurlyClose))
            .labelled("'node body'");

        any_doc_comment
            .repeated()
            .collect()
            .then(any_ident.labelled("'node-type'"))
            .then(any_ident.labelled("'node-name'").or_not())
            .then(expression.repeated().collect::<Vec<_>>())
            .then(type_specifier.or_not())
            .then(node_body.or_not())
            .map_with(
                |(((((doc_comments, node_type), name), expressions), type_specifier), body),
                 extra| {
                    let (properties, sub_nodes) = body.unwrap_or_default();

                    Node {
                        doc_comments,
                        node_type,
                        name,
                        type_specifier,
                        properties: expressions
                            .into_iter()
                            .map(|expression| {
                                let span = expression.span;
                                Property {
                                    name: None,
                                    expression,
                                }
                                .spanned(span)
                            })
                            .chain(properties)
                            .collect(),
                        sub_nodes,
                        span: extra.span(),
                    }
                },
            )
    });

    node.repeated().collect()
}
