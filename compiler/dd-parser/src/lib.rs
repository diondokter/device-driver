use std::{borrow::Cow, fmt::Display, num::NonZeroU32, sync::LazyLock};

use chumsky::{
    IterParser, Parser,
    error::Rich,
    extra,
    input::{Input, MappedInput},
    inspector::Inspector,
    prelude::{choice, just, recursive},
    select,
};
use device_driver_common::{
    interner::{Istr, StrExt},
    span::{Span, SpanExt, Spanned},
    specifiers::{Access, AddressMode, BaseType, ByteOrder, Integer},
};
use device_driver_diagnostics::{Diagnostics, errors::ParsingError};
use device_driver_lexer::{
    Token, TokenModifier, TokenType,
    semantic_token_object::{SemanticToken, SemanticTokenObject},
};

use crate::parse_num::{ParseIntRadix, ParseIntRadixError, ParseIntRadixErrorKind, parse_num};

#[cfg(feature = "gen-docs")]
pub mod gen_docs;
mod parse_num;

#[derive(Default)]
pub struct AstArena {
    nodes: Vec<Node>,
}

impl AstArena {
    pub fn alloc_node(&mut self, node: Node) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(node);
        id
    }
}

impl<'src, I: Input<'src>> Inspector<'src, I> for AstArena {
    type Checkpoint = AstArenaCheckpoint;

    fn on_token(&mut self, _token: &I::Token) {}

    fn on_save<'parse>(
        &self,
        _cursor: &chumsky::input::Cursor<'src, 'parse, I>,
    ) -> Self::Checkpoint {
        AstArenaCheckpoint {
            node_count: self.nodes.len(),
        }
    }

    fn on_rewind<'parse>(
        &mut self,
        marker: &chumsky::input::Checkpoint<'src, 'parse, I, Self::Checkpoint>,
    ) {
        // undo all the pushes that happened since the checkpoint
        self.nodes.truncate(marker.inspector().node_count);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AstArenaCheckpoint {
    node_count: usize,
}

pub fn parse(tokens: &[Spanned<Token>], diagnostics: &mut Diagnostics) -> Ast {
    let mut state = AstArena { nodes: Vec::new() };

    let (ast, parse_errs) = node()
        .map_with(|ast, e| (ast, e.span()))
        .parse_with_state(
            tokens.map(
                tokens
                    .last()
                    .map(|t| Span::from(t.span.end..t.span.end))
                    .unwrap_or_default(),
                |token| (&token.value, &token.span),
            ),
            &mut state,
        )
        .into_output_errors();

    for error in parse_errs {
        diagnostics.add(ParsingError {
            reason: error.to_string(),
            span: *error.span(),
        });
    }

    Ast::new(
        ast.map(|(ast, _)| ast),
        state,
        ast.as_ref().map(|(_, span)| *span).unwrap_or_default(),
    )
}

// Don't forget to update the book when parsers are added, changed or removed!
#[derive(Debug)]
pub struct Ast {
    pub root_node: Option<NodeId>,
    nodes: Vec<Node>,
    pub span: Span,
}

impl Ast {
    pub fn new(root_node: Option<NodeId>, arena: AstArena, span: Span) -> Self {
        Self {
            root_node,
            nodes: arena.nodes,
            span,
        }
    }

    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id.0]
    }

    pub fn node_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id.0]
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut [Node] {
        &mut self.nodes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeId(usize);

#[derive(Debug, Clone)]
pub struct Node {
    pub doc_comments: Vec<Spanned<Istr>>,
    pub node_type: Ident,
    pub name: Ident,
    pub repeat: Option<Spanned<Repeat>>,
    pub type_specifier: Option<Spanned<TypeSpecifier>>,
    pub short_properties: Vec<Spanned<Expression>>,
    pub properties: Vec<Spanned<Property>>,
    pub sub_nodes: Vec<NodeId>,
    pub span: Span,
}

impl Node {
    pub fn to_string_formatted(&self, ast: &Ast) -> String {
        let mut buf = String::new();
        self.fmt_formatted(&mut buf, ast, 0).unwrap();
        buf
    }

    pub fn fmt_formatted(
        &self,
        f: &mut impl std::fmt::Write,
        ast: &Ast,
        indentation_level: usize,
    ) -> std::fmt::Result {
        let indentation = format!("{:width$}", "", width = indentation_level * 4);

        for doc_comment in &self.doc_comments {
            writeln!(
                f,
                "{indentation}///{}{doc_comment}",
                if doc_comment.starts_with(" ") {
                    ""
                } else {
                    " "
                }
            )?;
        }
        write!(f, "{indentation}{} {}", self.node_type.val, self.name.val)?;

        if let Some(repeat) = self.repeat {
            write!(f, "[{} stride {}]", repeat.source, repeat.stride)?;
        }

        for expression in self.short_properties.iter() {
            write!(f, " {}", expression.print_formatted(ast))?;
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
                        let node = ast.node(*node);
                        if node.doc_comments.is_empty() {
                            for (i, line) in node.to_string_formatted(ast).lines().enumerate() {
                                if i == 0 {
                                    write!(f, " {line}")?;
                                } else {
                                    write!(f, "\n{indentation}{line}")?;
                                }
                            }
                        } else {
                            node.fmt_formatted(f, ast, indentation_level + 1)?;
                        }
                    }
                }
            }
        }

        if !self.sub_nodes.is_empty() || !self.properties.is_empty() {
            writeln!(f, " {{")?;

            for property in self.properties.iter() {
                for doc_comment in property.doc_comments.iter() {
                    writeln!(
                        f,
                        "{indentation}    ///{}{}",
                        if doc_comment.starts_with(" ") {
                            ""
                        } else {
                            " "
                        },
                        doc_comment
                    )?;
                }

                write!(f, "{indentation}    {}:", property.name.val)?;

                let expression = property.expression.print_formatted(ast);

                if expression.starts_with("///") {
                    for line in expression.lines() {
                        write!(f, "\n{indentation}        {line}")?;
                    }
                } else {
                    for (i, line) in expression.lines().enumerate() {
                        if i == 0 {
                            write!(f, " {line}")?;
                        } else {
                            write!(f, "\n{indentation}    {line}")?;
                        }
                    }
                }

                writeln!(f, ",")?;
            }

            if !self.properties.is_empty() && !self.sub_nodes.is_empty() {
                writeln!(f, "{indentation}",)?;
            }

            for node in self.sub_nodes.iter() {
                let node = ast.node(*node);
                node.fmt_formatted(f, ast, indentation_level + 1)?;
            }

            write!(f, "{indentation}}}")?;
        }

        Ok(())
    }
}

impl SemanticTokenObject for Node {
    type Context = Ast;

    fn to_tokens_in(&self, ctx: &Self::Context, tokens: &mut Vec<SemanticToken<'_>>) {
        tokens.extend(self.doc_comments.iter().map(|doc_comment| {
            Token::DocCommentLine(doc_comment.as_str())
                .with_semantics(TokenType::Comment, &[TokenModifier::Documentation])
        }));
        tokens.push(
            Token::Ident(self.node_type.val.as_str()).with_semantics(TokenType::NodeType, &[]),
        );
        tokens.push(Token::Ident(self.name.val.as_str()).with_semantics(
            TokenType::Type,
            &[TokenModifier::Declaration, TokenModifier::Definition],
        ));

        if let Some(repeat) = self.repeat.as_ref() {
            repeat.to_tokens_in(&(), tokens);
        }

        for short_property in self.short_properties.iter() {
            short_property.to_tokens_in(ctx, tokens);
        }

        if let Some(type_specifier) = self.type_specifier.as_ref() {
            type_specifier.to_tokens_in(ctx, tokens);
        }

        if self.properties.len() + self.sub_nodes.len() > 0 {
            tokens.push(Token::CurlyOpen.without_semantics());

            for property in self.properties.iter() {
                property.to_tokens_in(ctx, tokens);
                tokens.push(Token::Comma.without_semantics());
            }

            for sub_node in self.sub_nodes.iter() {
                ctx.node(*sub_node).to_tokens_in(ctx, tokens);
                tokens.push(Token::Comma.without_semantics());
            }

            tokens.push(Token::CurlyClose.without_semantics());
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeSpecifier {
    pub base_type: Spanned<BaseType>,
    pub use_try: bool,
    pub conversion: Option<TypeConversion>,
}

impl SemanticTokenObject for TypeSpecifier {
    type Context = Ast;

    fn to_tokens_in(&self, ctx: &Self::Context, tokens: &mut Vec<SemanticToken<'_>>) {
        tokens.push(Token::Arrow.without_semantics());
        self.base_type.to_tokens_in(&(), tokens);
        if let Some(conversion) = self.conversion.as_ref() {
            tokens.push(Token::As.with_semantics(TokenType::Keyword, &[]));
            if self.use_try {
                tokens.push(Token::Try.with_semantics(TokenType::Keyword, &[]));
            }
            conversion.to_tokens_in(ctx, tokens);
        }
    }
}

#[derive(Debug, Clone)]
pub enum TypeConversion {
    Reference(Ident),
    Subnode(NodeId),
}

impl TypeConversion {
    pub fn as_subnode(&self) -> Option<NodeId> {
        if let Self::Subnode(v) = self {
            Some(*v)
        } else {
            None
        }
    }
}

impl SemanticTokenObject for TypeConversion {
    type Context = Ast;

    fn to_tokens_in(&self, ctx: &Self::Context, tokens: &mut Vec<SemanticToken<'_>>) {
        match self {
            TypeConversion::Reference(ident) => {
                tokens.push(Token::Ident(ident.val.as_str()).with_semantics(TokenType::Type, &[]))
            }
            TypeConversion::Subnode(node) => ctx.node(*node).to_tokens_in(ctx, tokens),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Property {
    pub doc_comments: Vec<Spanned<Istr>>,
    pub name: Ident,
    pub expression: Spanned<Expression>,
}

impl SemanticTokenObject for Property {
    type Context = Ast;

    fn to_tokens_in(&self, ctx: &Self::Context, tokens: &mut Vec<SemanticToken<'_>>) {
        tokens.extend(self.doc_comments.iter().map(|istr| {
            Token::DocCommentLine(istr.as_str())
                .with_semantics(TokenType::Comment, &[TokenModifier::Documentation])
        }));
        tokens.push(Token::Ident(self.name.val.as_str()).with_semantics(TokenType::Property, &[]));

        tokens.push(Token::Colon.without_semantics());
        self.expression.to_tokens_in(ctx, tokens);
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    AddressRange { end: i128, start: i128 },
    ByteArray(Vec<u8>),
    BaseType(BaseType),
    Integer(Integer),
    Allow,
    Number(i128),
    DefaultNumber(Option<i128>),
    CatchAllNumber(Option<i128>),
    String(Istr),
    Access(Access),
    ByteOrder(ByteOrder),
    TypeReference(Ident),
    SubNode(NodeId),
    Auto,
    AddressMode(AddressMode),
}

impl Expression {
    pub fn as_range(&self) -> Option<(i128, i128)> {
        if let Self::AddressRange { end, start } = self {
            Some((*end, *start))
        } else {
            None
        }
    }

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

    pub fn as_unsigned_integer(&self) -> Option<Integer> {
        if let Self::Integer(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_number(&self) -> Option<i128> {
        if let Self::Number(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<Istr> {
        if let Self::String(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_address_mode(&self) -> Option<AddressMode> {
        if let Self::AddressMode(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_sub_node(&self) -> Option<NodeId> {
        if let Self::SubNode(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn print_formatted(&self, ast: &Ast) -> Cow<'static, str> {
        match self {
            Expression::AddressRange { end, start } => format!("{end}:{start}").into(),
            Expression::ByteArray(items) => format!("{items:?}").into(),
            Expression::BaseType(base_type) => base_type.to_string().into(),
            Expression::Integer(integer) => integer.to_string().into(),
            Expression::Allow => "allow".into(),
            Expression::Number(num) => num.to_string().into(),
            Expression::DefaultNumber(Some(num)) => format!("default {num}").into(),
            Expression::DefaultNumber(None) => "default _".into(),
            Expression::CatchAllNumber(Some(num)) => format!("catch-all {num}").into(),
            Expression::CatchAllNumber(None) => "catch-all _".into(),
            Expression::String(val) => format!("\"{val}\"").into(),
            Expression::Access(val) => val.to_string().into(),
            Expression::ByteOrder(val) => val.to_string().into(),
            Expression::TypeReference(ident) => ident.val.to_string().into(),
            Expression::SubNode(val) => ast.node(*val).to_string_formatted(ast).into(),
            Expression::Auto => "_".into(),
            Expression::AddressMode(val) => val.to_string().into(),
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::AddressRange { .. } => write!(f, "range"),
            Expression::ByteArray(_) => write!(f, "[bytes]"),
            Expression::BaseType(_) => write!(f, "base type"),
            Expression::Integer(_) => write!(f, "integer type"),
            Expression::Allow => write!(f, "allow"),
            Expression::Number(_) => write!(f, "number"),
            Expression::DefaultNumber(None) => write!(f, "default auto"),
            Expression::CatchAllNumber(None) => write!(f, "catch-all auto"),
            Expression::DefaultNumber(Some(_)) => write!(f, "default number"),
            Expression::CatchAllNumber(Some(_)) => write!(f, "catch-all number"),
            Expression::String(_) => write!(f, "string"),
            Expression::Access(_) => write!(f, "access specifier"),
            Expression::ByteOrder(_) => write!(f, "byte order"),
            Expression::TypeReference(_) => write!(f, "type reference"),
            Expression::SubNode(_) => write!(f, "sub node"),
            Expression::Auto => write!(f, "auto"),
            Expression::AddressMode(_) => write!(f, "address mode"),
        }
    }
}

impl SemanticTokenObject for Expression {
    type Context = Ast;

    fn to_tokens_in(&self, ctx: &Self::Context, tokens: &mut Vec<SemanticToken<'_>>) {
        match self {
            Expression::AddressRange { end, start } => {
                tokens.push(
                    Token::Num(end.to_string().intern().as_str())
                        .with_semantics(TokenType::Number, &[]),
                );
                tokens.push(Token::Colon.without_semantics());
                tokens.push(
                    Token::Num(start.to_string().intern().as_str())
                        .with_semantics(TokenType::Number, &[]),
                );
            }
            Expression::ByteArray(bytes) => {
                tokens.push(Token::BracketOpen.without_semantics());
                for (i, byte) in bytes.iter().enumerate() {
                    if i != 0 {
                        tokens.push(Token::Comma.without_semantics());
                    }
                    tokens.push(
                        Token::Num(byte.to_string().intern().as_str())
                            .with_semantics(TokenType::Number, &[]),
                    );
                }
                tokens.push(Token::BracketClose.without_semantics());
            }
            Expression::BaseType(base_type) => base_type.to_tokens_in(&(), tokens),
            Expression::Integer(integer) => integer.to_tokens_in(&(), tokens),
            Expression::Allow => tokens.push(Token::Allow.with_semantics(TokenType::Keyword, &[])),
            Expression::Number(num) => tokens.push(
                Token::Num(num.to_string().intern().as_str())
                    .with_semantics(TokenType::Number, &[]),
            ),
            Expression::DefaultNumber(num) => {
                tokens.push(Token::Default.with_semantics(TokenType::Keyword, &[]));
                match num {
                    Some(num) => {
                        tokens.push(
                            Token::Num(num.to_string().intern().as_str())
                                .with_semantics(TokenType::Number, &[]),
                        );
                    }
                    None => {
                        tokens.push(Token::Underscore.with_semantics(TokenType::Operator, &[]));
                    }
                }
            }
            Expression::CatchAllNumber(num) => {
                tokens.push(Token::CatchAll.with_semantics(TokenType::Keyword, &[]));
                match num {
                    Some(num) => {
                        tokens.push(
                            Token::Num(num.to_string().intern().as_str())
                                .with_semantics(TokenType::Number, &[]),
                        );
                    }
                    None => {
                        tokens.push(Token::Underscore.with_semantics(TokenType::Operator, &[]));
                    }
                }
            }
            Expression::String(istr) => {
                tokens.push(Token::String(istr.as_str()).with_semantics(TokenType::String, &[]));
            }
            Expression::Access(access) => access.to_tokens_in(&(), tokens),
            Expression::ByteOrder(byte_order) => byte_order.to_tokens_in(&(), tokens),
            Expression::TypeReference(ident) => {
                tokens.push(Token::Ident(ident.val.as_str()).with_semantics(TokenType::Type, &[]))
            }
            Expression::SubNode(node) => ctx.node(*node).to_tokens_in(ctx, tokens),
            Expression::Auto => {
                tokens.push(Token::Underscore.with_semantics(TokenType::Operator, &[]));
            }
            Expression::AddressMode(address_mode) => address_mode.to_tokens_in(&(), tokens),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Repeat {
    pub source: Spanned<RepeatSource>,
    pub stride: Spanned<i32>,
}

impl SemanticTokenObject for Repeat {
    type Context = ();

    fn to_tokens_in(&self, ctx: &Self::Context, tokens: &mut Vec<SemanticToken<'_>>) {
        tokens.push(Token::BracketOpen.without_semantics());
        self.source.to_tokens_in(ctx, tokens);
        tokens.push(Token::Stride.with_semantics(TokenType::Keyword, &[]));
        tokens.push(
            Token::Num(self.stride.to_string().intern().as_str())
                .with_semantics(TokenType::Number, &[]),
        );
        tokens.push(Token::BracketClose.without_semantics());
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RepeatSource {
    Count(NonZeroU32),
    Enum(Ident),
}

impl Default for RepeatSource {
    fn default() -> Self {
        Self::Count(1.try_into().unwrap())
    }
}

impl Display for RepeatSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepeatSource::Count(non_zero) => write!(f, "{non_zero}"),
            RepeatSource::Enum(ident) => write!(f, "{}", ident.val),
        }
    }
}

impl SemanticTokenObject for RepeatSource {
    type Context = ();

    fn to_tokens_in(&self, _ctx: &Self::Context, tokens: &mut Vec<SemanticToken<'_>>) {
        match self {
            RepeatSource::Count(non_zero) => tokens.push(
                Token::Num(non_zero.get().to_string().intern().as_str())
                    .with_semantics(TokenType::Number, &[]),
            ),
            RepeatSource::Enum(ident) => {
                tokens.push(Token::Ident(ident.val.as_str()).with_semantics(TokenType::Type, &[]))
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ident {
    pub val: Istr,
    pub span: Span,
}

impl Ident {
    pub const fn new(val: Istr, span: Span) -> Self {
        Self { val, span }
    }

    pub const fn new_no_span(val: Istr) -> Self {
        Self {
            val,
            span: Span::empty(),
        }
    }

    pub fn new_auto(span: Span) -> Self {
        Self {
            val: "_".intern(),
            span,
        }
    }

    /// Returns true if the identifier was specified using an underscore token
    pub fn is_auto(&self) -> bool {
        static UNDERSCORE: LazyLock<Istr> = LazyLock::new(|| "_".intern());
        self.val == *UNDERSCORE
    }
}

fn try_num<'tokens, 'src: 'tokens, I: ParseIntRadix>(
    num_str: &'src str,
    span: Span,
) -> Result<I, RichErr<'tokens, 'src>> {
    match parse_num::<I>(num_str) {
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
                    "number `{source}` is parsed as a {}{target_bits}, but overflows.",
                    if target_signed { 'i' } else { 'u' }
                ),
            )),
            ParseIntRadixErrorKind::Underflow => Err(Rich::custom(
                span,
                format!(
                    "number `{source}` is parsed as a {}{target_bits}, but underflows.",
                    if target_signed { 'i' } else { 'u' }
                ),
            )),
            ParseIntRadixErrorKind::Empty => Err(Rich::custom(
                span,
                format!("could not parse `{source}` as a number because it contains no numbers"),
            )),
            ParseIntRadixErrorKind::Zero => {
                Err(Rich::custom(span, "number can't be 0 in this position"))
            }
        },
    }
}

pub type InputType<'tokens, 'src> =
    MappedInput<'tokens, Token<'src>, Span, &'tokens [Spanned<Token<'src>>]>;
pub type RichErr<'tokens, 'src> = Rich<'tokens, Token<'src>, Span>;
pub type RichExtra<'tokens, 'src> = extra::Full<RichErr<'tokens, 'src>, AstArena, ()>;

pub fn ident<'tokens, 'src: 'tokens>(
    allow_auto: bool,
) -> impl Parser<'tokens, InputType<'tokens, 'src>, Ident, RichExtra<'tokens, 'src>> + Clone {
    select! {
        Token::Ident(val) = e => Ident::new(val.intern(), e.span()),
        Token::Underscore = e if allow_auto => Ident::new_auto(e.span()),
    }
    .labelled(format!(
        "Ident{}",
        if allow_auto { "|Underscore" } else { "" }
    ))
    .as_terminal()
}

pub fn doc_comment<'tokens, 'src: 'tokens>()
-> impl Parser<'tokens, InputType<'tokens, 'src>, Spanned<Istr>, RichExtra<'tokens, 'src>> + Copy {
    select! {
        Token::DocCommentLine(val) => val
    }
    .map_with(|line, extra| line.intern().spanned(extra.span()))
    .labelled("DocCommentLine")
    .as_terminal()
}

pub fn num<'tokens, 'src: 'tokens, I: ParseIntRadix>()
-> impl Parser<'tokens, InputType<'tokens, 'src>, I, RichExtra<'tokens, 'src>> + Clone {
    select! {
        Token::Num(num) => num
    }
    .try_map(try_num::<I>)
    .labelled(format!(
        "Num<{}>",
        // Get the type name of the integer, excluding module path
        std::any::type_name::<I>().split("::").last().unwrap()
    ))
    .as_terminal()
}

pub fn range<'tokens, 'src: 'tokens>()
-> impl Parser<'tokens, InputType<'tokens, 'src>, Expression, RichExtra<'tokens, 'src>> + Clone {
    num::<i128>()
        .then_ignore(just(Token::Colon))
        .then(num::<i128>())
        .map(|(end, start)| Expression::AddressRange { end, start })
        .labelled("range")
}

pub fn base_type<'tokens, 'src: 'tokens>()
-> impl Parser<'tokens, InputType<'tokens, 'src>, BaseType, RichExtra<'tokens, 'src>> + Copy {
    select! { Token::BaseType(bt) => bt }
        .labelled("BaseType")
        .as_terminal()
}

pub fn integer<'tokens, 'src: 'tokens>()
-> impl Parser<'tokens, InputType<'tokens, 'src>, Integer, RichExtra<'tokens, 'src>> + Copy {
    select! { Token::Integer(i) => i }
        .labelled("Integer")
        .as_terminal()
}

pub fn byte_array<'tokens, 'src: 'tokens>()
-> impl Parser<'tokens, InputType<'tokens, 'src>, Expression, RichExtra<'tokens, 'src>> + Clone {
    num::<u8>()
        .separated_by(just(Token::Comma))
        .collect::<Vec<_>>()
        .map(Expression::ByteArray)
        .then_ignore(just(Token::Comma).or_not())
        .delimited_by(just(Token::BracketOpen), just(Token::BracketClose))
        .labelled("byte-array")
}

/// Expression without type reference since that clashes with nodes
pub fn simple_expression<'tokens, 'src: 'tokens>()
-> impl Parser<'tokens, InputType<'tokens, 'src>, Spanned<Expression>, RichExtra<'tokens, 'src>> + Clone
{
    choice((
        range().labelled("range").as_non_terminal(),
        base_type().map(Expression::BaseType),
        integer().map(Expression::Integer),
        num::<i128>().map(Expression::Number),
        just(Token::Default)
            .ignore_then(
                num::<i128>()
                    .map(Some)
                    .or(just(Token::Underscore).map(|_| None)),
            )
            .map(Expression::DefaultNumber)
            .labelled("default-number"),
        just(Token::CatchAll)
            .ignore_then(
                num::<i128>()
                    .map(Some)
                    .or(just(Token::Underscore).map(|_| None)),
            )
            .map(Expression::CatchAllNumber)
            .labelled("catch-all-number"),
        byte_array().labelled("byte-array").as_non_terminal(),
        just(Token::Allow).map(|_| Expression::Allow),
        select! { Token::Access(val) => val }
            .map(Expression::Access)
            .labelled("Access")
            .as_terminal(),
        select! { Token::ByteOrder(val) => val }
            .map(Expression::ByteOrder)
            .labelled("ByteOrder")
            .as_terminal(),
        just(Token::Underscore).map(|_| Expression::Auto),
        select! { Token::String(val) => val.intern() }
            .map(Expression::String)
            .labelled("String")
            .as_terminal(),
        select! { Token::AddressMode(val) => val }
            .map(Expression::AddressMode)
            .labelled("AddressMode")
            .as_terminal(),
    ))
    .map_with(|expression, extra| expression.spanned(extra.span()))
    .labelled("simple-expression")
}

pub fn repeat<'tokens, 'src: 'tokens>()
-> impl Parser<'tokens, InputType<'tokens, 'src>, Spanned<Repeat>, RichExtra<'tokens, 'src>> + Clone
{
    choice((
        num::<NonZeroU32>().map(RepeatSource::Count),
        ident(false).map(RepeatSource::Enum),
    ))
    .map_with(|repeat_source, extra| repeat_source.with_span(extra.span()))
    .then(
        just(Token::Stride)
            .ignore_then(num::<i32>().map_with(|num, extra| num.with_span(extra.span()))),
    )
    .delimited_by(just(Token::BracketOpen), just(Token::BracketClose))
    .map_with(|(source, stride), extra| Repeat { source, stride }.spanned(extra.span()))
    .labelled("repeat")
}

pub fn property<'tokens, 'src: 'tokens, 'node>(
    node: impl Parser<'tokens, InputType<'tokens, 'src>, NodeId, RichExtra<'tokens, 'src>> + Clone,
) -> impl Parser<'tokens, InputType<'tokens, 'src>, Spanned<Property>, RichExtra<'tokens, 'src>> + Clone
{
    doc_comment()
        .repeated()
        .collect()
        .then(
            ident(false)
                .then(
                    just(Token::Colon).ignore_then(choice((
                        simple_expression()
                            .labelled("simple-expression")
                            .as_non_terminal(),
                        node.clone()
                            .map_with(|node, extra| Expression::SubNode(node).spanned(extra.span()))
                            .labelled("node")
                            .as_non_terminal(),
                        ident(false)
                            .map(Expression::TypeReference)
                            .map_with(|expression, extra| expression.spanned(extra.span())),
                    ))),
                )
                .map_with(|(name, expression), extra| {
                    Property {
                        doc_comments: Vec::new(),
                        name,
                        expression,
                    }
                    .spanned(extra.span())
                }),
        )
        .map(|(docs, mut prop)| {
            prop.doc_comments = docs;
            prop
        })
        .labelled("property")
}

pub fn type_specifier<'tokens, 'src: 'tokens>(
    node: impl Parser<'tokens, InputType<'tokens, 'src>, NodeId, RichExtra<'tokens, 'src>> + Clone,
) -> impl Parser<'tokens, InputType<'tokens, 'src>, Spanned<TypeSpecifier>, RichExtra<'tokens, 'src>>
+ Clone {
    let type_conversion = just(Token::As).ignore_then(just(Token::Try).or_not()).then(
        node.labelled("node")
            .as_non_terminal()
            .map(TypeConversion::Subnode)
            .or(ident(false).map(TypeConversion::Reference)),
    );
    just(Token::Arrow)
        .ignore_then(
            choice((
                base_type(),
                integer().map(BaseType::FixedSize),
                just(Token::Underscore).map(|_| BaseType::Unspecified),
            ))
            .map_with(|b, e| b.spanned(e.span())),
        )
        .then(type_conversion.or_not())
        .map(|(base_type, conversion)| TypeSpecifier {
            base_type,
            use_try: conversion
                .as_ref()
                .map(|(try_token, _)| try_token.is_some())
                .unwrap_or_default(),
            conversion: conversion.map(|(_, conversion)| conversion),
        })
        .map_with(|ts, e| ts.spanned(e.span()))
        .labelled("type-specifier")
}

pub fn node_body<'tokens, 'src: 'tokens>(
    node: impl Parser<'tokens, InputType<'tokens, 'src>, NodeId, RichExtra<'tokens, 'src>> + Clone,
) -> impl Parser<
    'tokens,
    InputType<'tokens, 'src>,
    (Vec<Spanned<Property>>, Vec<NodeId>),
    RichExtra<'tokens, 'src>,
> + Clone {
    let properties = property(node.clone())
        .labelled("property")
        .as_non_terminal()
        .separated_by(just(Token::Comma))
        .at_least(1)
        .collect::<Vec<_>>();
    let nodes = node
        .labelled("node")
        .as_non_terminal()
        .separated_by(just(Token::Comma))
        .at_least(1)
        .collect::<Vec<_>>();

    // Body with comma forced between properties and nodes
    choice((
        // Properties + comma + nodes
        properties
            .clone()
            .then_ignore(just(Token::Comma))
            .then(nodes.clone()),
        // Properties + no comma + no nodes
        properties
            .clone()
            .map(|properties| (properties, Vec::new())),
        // No properties + no comma + nodes
        nodes.map(|nodes| (Vec::new(), nodes)),
    ))
    .then_ignore(just(Token::Comma).or_not())
    .or_not()
    .map(|body| body.unwrap_or_default())
    .delimited_by(just(Token::CurlyOpen), just(Token::CurlyClose))
    .labelled("node-body")
}

pub fn node<'tokens, 'src: 'tokens>()
-> impl Parser<'tokens, InputType<'tokens, 'src>, NodeId, RichExtra<'tokens, 'src>> + Clone {
    recursive(|node| {
        let node = node.labelled("node").as_non_terminal();

        doc_comment()
            .repeated()
            .collect()
            .then(ident(false).labelled("node-type"))
            .then(ident(true).labelled("node-name"))
            .then(repeat().labelled("repeat").as_non_terminal().or_not())
            .then(
                simple_expression()
                    .labelled("simple-expression")
                    .as_non_terminal()
                    .repeated()
                    .collect::<Vec<_>>(),
            )
            .then(
                type_specifier(node.clone())
                    .labelled("type-specifier")
                    .as_non_terminal()
                    .or_not(),
            )
            .then(
                node_body(node.clone())
                    .labelled("node-body")
                    .as_non_terminal()
                    .or_not(),
            )
            .map_with(
                |(
                    (((((doc_comments, node_type), name), repeat), expressions), type_specifier),
                    body,
                ),
                 extra| {
                    let (properties, sub_nodes) = body.unwrap_or_default();

                    let mut span: Span = extra.span();
                    span = span.start_from(node_type.span);

                    extra.state().alloc_node(Node {
                        doc_comments,
                        node_type,
                        name,
                        repeat,
                        type_specifier,
                        properties,
                        short_properties: expressions,
                        sub_nodes,
                        span,
                    })
                },
            )
            .labelled("node")
    })
}
