#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::VariantNames, strum::Display, strum::EnumString, Hash,
)]
#[strum(serialize_all = "lowercase")]
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

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    strum::VariantNames,
    strum::Display,
    strum::EnumString,
    Hash,
)]
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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::VariantNames, strum::Display, strum::EnumString, Hash,
)]
pub enum ByteOrder {
    LE,
    BE,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    strum::VariantNames,
    strum::Display,
    strum::EnumString,
    Hash,
)]
pub enum BitOrder {
    #[default]
    LSB0,
    MSB0,
}
