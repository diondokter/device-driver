use std::{borrow::Cow, fmt::Display, num::IntErrorKind};

use device_driver_common::{
    span::{SpanExt, Spanned},
    specifiers::{Access, BaseType, ByteOrder, Integer},
};
use logos::Logos;

pub fn lex<'src>(source: &'src str) -> Vec<Spanned<Token<'src>>> {
    Token::lexer(source)
        .spanned()
        .map(|(token, span)| match token {
            Ok(token) => token.with_span(span),
            Err(()) => Token::Error.with_span(span),
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Logos)]
#[logos(skip r"[ \t\r\n]+")] // Skip (common) whitespace
#[logos(skip(r"//[^\n]*", allow_greedy = true))] // Skip comments
pub enum Token<'src> {
    #[regex(r"///[^\n]*", allow_greedy = true, callback = |lex| lex.slice().trim_start_matches("///"))]
    DocCommentLine(&'src str),
    #[regex(r"\p{XID_Start}[\p{XID_Continue}-]*")]
    Ident(&'src str),
    #[token("{")]
    CurlyOpen,
    #[token("}")]
    CurlyClose,
    #[token("[")]
    BracketOpen,
    #[token("]")]
    BracketClose,
    #[token("<")]
    AngleOpen,
    #[token(">")]
    AngleClose,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("_")]
    Underscore,
    #[token("->")]
    Arrow,
    #[token("by")]
    By,
    #[token("try")]
    Try,
    #[token("as")]
    As,
    #[token("allow")]
    Allow,
    #[token("default")]
    Default,
    #[token("catch-all")]
    CatchAll,
    #[regex(r"-?[0-9][_0-9]*")] // Decimal
    #[regex(r"-?0b[_0-1]+")] // Binary, octal & hex
    #[regex(r"-?0o[_0-7]+")] // Binary, octal & hex
    #[regex(r"-?0x[_0-9a-fA-F]+")] // Binary, octal & hex
    Num(&'src str),
    #[token("RW", |_| Access::RW)]
    #[token("RO", |_| Access::RO)]
    #[token("WO", |_| Access::WO)]
    Access(Access),
    #[token("BE", |_| ByteOrder::BE)]
    #[token("LE", |_| ByteOrder::LE)]
    ByteOrder(ByteOrder),
    /// All the base types except fixed integers. Those are [Self::Integer].
    #[token("uint", |_| BaseType::Uint)]
    #[token("int", |_| BaseType::Int)]
    #[token("bool", |_| BaseType::Bool)]
    BaseType(BaseType),
    #[token("u8", |_| Integer::U8)]
    #[token("u16", |_| Integer::U16)]
    #[token("u32", |_| Integer::U32)]
    #[token("u64", |_| Integer::U64)]
    #[token("i8", |_| Integer::I8)]
    #[token("i16", |_| Integer::I16)]
    #[token("i32", |_| Integer::I32)]
    #[token("i64", |_| Integer::I64)]
    Integer(Integer),
    #[regex(r"\S", priority = 0)] // Any non-whitespace character
    Unexpected(&'src str),
    Error, // Catch-all for errors
}

impl<'src> Display for Token<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::DocCommentLine(_) => write!(f, "doc comment"),
            Token::Ident(_) => write!(f, "identifier"),
            Token::CurlyOpen => write!(f, "{{"),
            Token::CurlyClose => write!(f, "}}"),
            Token::BracketOpen => write!(f, "["),
            Token::BracketClose => write!(f, "]"),
            Token::AngleOpen => write!(f, "<"),
            Token::AngleClose => write!(f, ">"),
            Token::Colon => write!(f, ":"),
            Token::Underscore => write!(f, "_"),
            Token::Comma => write!(f, ","),
            Token::Arrow => write!(f, "->"),
            Token::By => write!(f, "by"),
            Token::Try => write!(f, "try"),
            Token::As => write!(f, "as"),
            Token::Allow => write!(f, "allow"),
            Token::Default => write!(f, "default"),
            Token::CatchAll => write!(f, "catch-all"),
            Token::Num(_) => write!(f, "number"),
            Token::Access(_) => write!(f, "access-specifier"),
            Token::ByteOrder(_) => write!(f, "byte-order"),
            Token::BaseType(_) => write!(f, "base type"),
            Token::Integer(_) => write!(f, "integer type"),
            Token::Unexpected(val) => write!(f, "{}", val.escape_debug()),
            Token::Error => write!(f, "ERROR"),
        }
    }
}

impl<'src> Token<'src> {
    fn get_human_string(&self) -> Cow<'static, str> {
        match self {
            Token::DocCommentLine(line) => format!("///{line}").into(),
            Token::Ident(ident) => format!("#{ident}").into(),
            Token::CurlyOpen => "{".into(),
            Token::CurlyClose => "}".into(),
            Token::BracketOpen => "[".into(),
            Token::BracketClose => "]".into(),
            Token::AngleOpen => "<".into(),
            Token::AngleClose => ">".into(),
            Token::Colon => ":".into(),
            Token::Underscore => "_".into(),
            Token::Comma => ",".into(),
            Token::Arrow => "->".into(),
            Token::Try => "try".into(),
            Token::By => "by".into(),
            Token::As => "as".into(),
            Token::Num(n) => n.to_string().into(),
            Token::Access(val) => val.to_string().into(),
            Token::ByteOrder(val) => val.to_string().into(),
            Token::BaseType(val) => val.to_string().into(),
            Token::Integer(val) => val.to_string().into(),
            Token::Allow => "allow".into(),
            Token::Default => "default".into(),
            Token::CatchAll => "catch-all".into(),
            Token::Unexpected(raw) => format!("!{raw}").into(),
            Token::Error => "UNEXPECTED".into(),
        }
    }

    /// New line before, new line after, indent change
    fn get_print_format(&self) -> (bool, bool, i32) {
        match self {
            Token::DocCommentLine(_) => (true, true, 0),
            Token::Comma => (false, true, 0),
            Token::CurlyOpen | Token::BracketOpen => (false, true, 1),
            Token::CurlyClose | Token::BracketClose => (true, false, -1),
            _ => (false, false, 0),
        }
    }

    pub fn formatted_print<'a, I: Iterator<Item = &'a Token<'src>>>(
        stream: &mut impl std::io::Write,
        tokens: I,
    ) -> std::io::Result<()>
    where
        'src: 'a,
    {
        let mut indent = 0i32;
        for token in tokens {
            let (newline_before, newline_after, indent_change) = token.get_print_format();

            indent += indent_change;
            if newline_before {
                write!(stream, "\n{:width$}", "", width = indent as usize * 4)?;
            }

            write!(stream, "{} ", token.get_human_string())?;

            if newline_after {
                write!(stream, "\n{:width$}", "", width = indent as usize * 4)?;
            }
        }

        Ok(())
    }

    pub fn parse_num<I: ParseIntRadix>(self) -> Result<I, ParseIntRadixError<'src>> {
        let Token::Num(num_slice) = self else {
            panic!("Token is not a number: `{:?}`", self);
        };

        let pos_num_slice = num_slice.trim_start_matches('-');
        let (mut cleaned_num_slice, radix) = match &pos_num_slice.get(0..2) {
            Some("0b") => (Cow::from(&pos_num_slice[2..]), 2),
            Some("0o") => (Cow::from(&pos_num_slice[2..]), 8),
            Some("0x") => (Cow::from(&pos_num_slice[2..]), 16),
            _ => (Cow::from(pos_num_slice), 10),
        };

        if cleaned_num_slice.contains('_') {
            cleaned_num_slice = cleaned_num_slice.replace("_", "").into();
        }

        if num_slice.starts_with('-') {
            cleaned_num_slice = ("-".to_string() + &cleaned_num_slice).into();
        }

        I::parse(num_slice, &cleaned_num_slice, radix)
    }
}

pub trait ParseIntRadix: Sized {
    fn parse<'src>(
        source: &'src str,
        cleaned_num_slice: &str,
        radix: u32,
    ) -> Result<Self, ParseIntRadixError<'src>>;
}

macro_rules! impl_parse_int_radix {
    ($int:ty) => {
        impl ParseIntRadix for $int {
            fn parse<'src>(
                source: &'src str,
                cleaned_num_slice: &str,
                radix: u32,
            ) -> Result<Self, ParseIntRadixError<'src>> {
                Self::from_str_radix(cleaned_num_slice, radix).map_err(|e| {
                    let kind = match e.kind() {
                        IntErrorKind::PosOverflow => ParseIntRadixErrorKind::Overflow,
                        IntErrorKind::NegOverflow => ParseIntRadixErrorKind::Underflow,
                        IntErrorKind::Empty => ParseIntRadixErrorKind::Empty,
                        _ => unreachable!("{e}"),
                    };

                    ParseIntRadixError {
                        source,
                        kind,
                        target_bits: Self::BITS,
                        target_signed: Self::MIN != 0,
                    }
                })
            }
        }
    };
}

impl_parse_int_radix!(u8);
impl_parse_int_radix!(u16);
impl_parse_int_radix!(u32);
impl_parse_int_radix!(u64);
impl_parse_int_radix!(u128);
impl_parse_int_radix!(i8);
impl_parse_int_radix!(i16);
impl_parse_int_radix!(i32);
impl_parse_int_radix!(i64);
impl_parse_int_radix!(i128);

pub struct ParseIntRadixError<'src> {
    pub source: &'src str,
    pub kind: ParseIntRadixErrorKind,
    pub target_bits: u32,
    pub target_signed: bool,
}

pub enum ParseIntRadixErrorKind {
    Overflow,
    Underflow,
    Empty,
}
