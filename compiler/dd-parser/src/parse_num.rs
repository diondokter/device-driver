use std::{
    borrow::Cow,
    num::{IntErrorKind, NonZeroU32},
};

pub fn parse_num<I: ParseIntRadix>(num_slice: &str) -> Result<I, ParseIntRadixError<'_>> {
    let pos_num_slice = num_slice.trim_start_matches('-');
    let (mut cleaned_num_slice, radix) = match &pos_num_slice.get(0..2) {
        Some("0b") => (Cow::from(&pos_num_slice[2..]), 2),
        Some("0o") => (Cow::from(&pos_num_slice[2..]), 8),
        Some("0x") => (Cow::from(&pos_num_slice[2..]), 16),
        _ => (Cow::from(pos_num_slice), 10),
    };

    if cleaned_num_slice.contains('_') {
        cleaned_num_slice = cleaned_num_slice.replace('_', "").into();
    }

    if num_slice.starts_with('-') {
        cleaned_num_slice = ("-".to_string() + &cleaned_num_slice).into();
    }

    I::parse(num_slice, &cleaned_num_slice, radix)
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
                        IntErrorKind::InvalidDigit if cleaned_num_slice.starts_with('-') => {
                            ParseIntRadixErrorKind::Underflow
                        }
                        _ => unreachable!("{e}: {cleaned_num_slice}"),
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

impl ParseIntRadix for NonZeroU32 {
    fn parse<'src>(
        source: &'src str,
        cleaned_num_slice: &str,
        radix: u32,
    ) -> Result<Self, ParseIntRadixError<'src>> {
        u32::from_str_radix(cleaned_num_slice, radix)
            .map_err(|e| match e.kind() {
                IntErrorKind::PosOverflow => ParseIntRadixErrorKind::Overflow,
                IntErrorKind::NegOverflow => ParseIntRadixErrorKind::Underflow,
                IntErrorKind::Empty => ParseIntRadixErrorKind::Empty,
                IntErrorKind::InvalidDigit if cleaned_num_slice.starts_with('-') => {
                    ParseIntRadixErrorKind::Underflow
                }
                _ => unreachable!("{e}: {cleaned_num_slice}"),
            })
            .and_then(|val| NonZeroU32::new(val).ok_or(ParseIntRadixErrorKind::Zero))
            .map_err(|kind| ParseIntRadixError {
                source,
                kind,
                target_bits: Self::BITS,
                target_signed: false,
            })
    }
}

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
    // A nonzero number is zero
    Zero,
}
