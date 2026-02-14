use std::{fmt::Display, str::FromStr};

use crate::{identifier::IdentifierRef, span::Spanned};

/// TODO: Remove when KDL is removed
pub trait VariantNames {
    /// Names of the variants of this enum
    const VARIANTS: &'static [&'static str];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Integer {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
}

impl VariantNames for Integer {
    const VARIANTS: &[&'static str] = &["u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64"];
}

impl Display for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::VARIANTS[*self as usize])
    }
}

impl FromStr for Integer {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "u8" => Ok(Self::U8),
            "u16" => Ok(Self::U16),
            "u32" => Ok(Self::U32),
            "u64" => Ok(Self::U64),
            "i8" => Ok(Self::I8),
            "i16" => Ok(Self::I16),
            "i32" => Ok(Self::I32),
            "i64" => Ok(Self::I64),
            _ => Err(()),
        }
    }
}

impl Integer {
    #[must_use]
    pub const fn is_signed(&self) -> bool {
        self.min_value() != 0
    }

    #[must_use]
    pub const fn min_value(&self) -> i128 {
        match self {
            Integer::U8 => u8::MIN as i128,
            Integer::U16 => u16::MIN as i128,
            Integer::U32 => u32::MIN as i128,
            Integer::U64 => u64::MIN as i128,
            Integer::I8 => i8::MIN as i128,
            Integer::I16 => i16::MIN as i128,
            Integer::I32 => i32::MIN as i128,
            Integer::I64 => i64::MIN as i128,
        }
    }

    #[must_use]
    pub const fn max_value(&self) -> i128 {
        match self {
            Integer::U8 => u8::MAX as i128,
            Integer::U16 => u16::MAX as i128,
            Integer::U32 => u32::MAX as i128,
            Integer::U64 => u64::MAX as i128,
            Integer::I8 => i8::MAX as i128,
            Integer::I16 => i16::MAX as i128,
            Integer::I32 => i32::MAX as i128,
            Integer::I64 => i64::MAX as i128,
        }
    }

    #[must_use]
    pub const fn size_bits(&self) -> u32 {
        match self {
            Integer::U8 => 8,
            Integer::U16 => 16,
            Integer::U32 => 32,
            Integer::U64 => 64,
            Integer::I8 => 8,
            Integer::I16 => 16,
            Integer::I32 => 32,
            Integer::I64 => 64,
        }
    }

    /// Find the smallest integer type that can fully contain the min and max
    /// and is equal or larger than the given `size_bits`.
    ///
    /// This function has a preference for unsigned integers.
    /// You can force a signed integer by making the min be negative (e.g. -1)
    #[must_use]
    pub const fn find_smallest(min: i128, max: i128, size_bits: u32) -> Option<Integer> {
        Some(match (min, max, size_bits) {
            (0.., ..0x1_00, ..=8) => Integer::U8,
            (0.., ..0x1_0000, ..=16) => Integer::U16,
            (0.., ..0x1_0000_0000, ..=32) => Integer::U32,
            (0.., ..0x1_0000_0000_0000_0000, ..=64) => Integer::U64,
            (-0x80.., ..0x80, ..=8) => Integer::I8,
            (-0x8000.., ..0x8000, ..=16) => Integer::I16,
            (-0x8000_00000.., ..0x8000_0000, ..=32) => Integer::I32,
            (-0x8000_0000_0000_0000.., ..0x8000_0000_0000_0000, ..=64) => Integer::I64,
            _ => return None,
        })
    }

    /// Given the min and the max and the sign of the integer,
    /// how many bits are required to fit the min and max? (inclusive)
    #[must_use]
    pub const fn bits_required(&self, min: i128, max: i128) -> u32 {
        assert!(max >= min);

        if self.is_signed() {
            let min_bits = if min.is_negative() {
                i128::BITS - (min.abs() - 1).leading_zeros() + 1
            } else {
                0
            };
            let max_bits = if max.is_positive() {
                i128::BITS - max.leading_zeros() + 1
            } else {
                0
            };

            if min_bits > max_bits {
                min_bits
            } else {
                max_bits
            }
        } else {
            assert!(min >= 0);
            i128::BITS - max.leading_zeros()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum Access {
    #[default]
    RW,
    RO,
    WO,
}

impl Access {
    #[must_use]
    pub fn is_readable(&self) -> bool {
        match self {
            Access::RW => true,
            Access::RO => true,
            Access::WO => false,
        }
    }
}

impl VariantNames for Access {
    const VARIANTS: &[&'static str] = &["RW", "RO", "WO"];
}

impl Display for Access {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::VARIANTS[*self as usize])
    }
}

impl FromStr for Access {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "RW" => Ok(Self::RW),
            "RO" => Ok(Self::RO),
            "WO" => Ok(Self::WO),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ByteOrder {
    LE,
    BE,
}

impl VariantNames for ByteOrder {
    const VARIANTS: &[&'static str] = &["LE", "BE"];
}

impl Display for ByteOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::VARIANTS[*self as usize])
    }
}

impl FromStr for ByteOrder {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LE" => Ok(Self::LE),
            "BE" => Ok(Self::BE),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum BaseType {
    Unspecified,
    Bool,
    #[default]
    Uint,
    Int,
    FixedSize(Integer),
}

impl BaseType {
    /// Returns `true` if the base type is [`Unspecified`].
    ///
    /// [`Unspecified`]: BaseType::Unspecified
    #[must_use]
    pub fn is_unspecified(&self) -> bool {
        matches!(self, Self::Unspecified)
    }

    /// Returns `true` if the base type is [`FixedSize`].
    ///
    /// [`FixedSize`]: BaseType::FixedSize
    #[must_use]
    pub fn is_fixed_size(&self) -> bool {
        matches!(self, Self::FixedSize(..))
    }
}

impl Display for BaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BaseType::Unspecified => write!(f, "unspecified"),
            BaseType::Bool => write!(f, "bool"),
            BaseType::Uint => write!(f, "uint"),
            BaseType::Int => write!(f, "int"),
            BaseType::FixedSize(integer) => write!(f, "{integer}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeConversion {
    /// The name of the type we're converting to
    pub type_name: Spanned<IdentifierRef>,
    /// True when we want to use the fallible interface (like a Result<type, error>)
    pub fallible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Repeat {
    pub source: RepeatSource,
    pub stride: i128,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RepeatSource {
    Count(u64),
    Enum(Spanned<IdentifierRef>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResetValue {
    Integer(u128),
    Array(Vec<u8>),
}

impl ResetValue {
    #[must_use]
    pub fn as_array(&self) -> Option<&Vec<u8>> {
        if let Self::Array(v) = self {
            Some(v)
        } else {
            None
        }
    }
}
