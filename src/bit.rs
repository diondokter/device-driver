/// A single bit
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Bit {
    /// The bit is set high
    Set = 0b1,
    /// The bit is set low
    Cleared = 0b0,
}

impl Bit {
    /// Returns true if the bit is set
    pub fn is_set(&self) -> bool {
        match self {
            Bit::Set => true,
            Bit::Cleared => false,
        }
    }

    /// Returns true if the bit is not set
    pub fn is_cleared(&self) -> bool {
        !self.is_set()
    }
}

impl From<bool> for Bit {
    fn from(val: bool) -> Self {
        match val {
            true => Bit::Set,
            false => Bit::Cleared,
        }
    }
}

impl From<Bit> for bool {
    fn from(val: Bit) -> Self {
        val.is_set()
    }
}

macro_rules! implement_int_conversion {
    ($int_type:ty) => {
        impl From<$int_type> for Bit {
            fn from(val: $int_type) -> Self {
                (val != 0).into()
            }
        }

        impl From<Bit> for $int_type {
            fn from(val: Bit) -> Self {
                val.is_set().into()
            }
        }
    };
}

implement_int_conversion!(u8);
implement_int_conversion!(u16);
implement_int_conversion!(u32);
implement_int_conversion!(u64);
implement_int_conversion!(u128);
implement_int_conversion!(i8);
implement_int_conversion!(i16);
implement_int_conversion!(i32);
implement_int_conversion!(i64);
implement_int_conversion!(i128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity() {
        assert_eq!(Bit::Set.is_set(), true);
        assert_eq!(Bit::Set.is_cleared(), false);

        assert_eq!(Bit::Cleared.is_set(), false);
        assert_eq!(Bit::Cleared.is_cleared(), true);

        assert_eq!(bool::from(Bit::Set), true);
        assert_eq!(bool::from(Bit::Cleared), false);

        assert_eq!(Bit::from(true), Bit::Set);
        assert_eq!(Bit::from(false), Bit::Cleared);
    }

    #[test]
    fn int_conversion() {
        assert_eq!(u8::from(Bit::Set), 1);
        assert_eq!(u8::from(Bit::Cleared), 0);
        assert_eq!(u16::from(Bit::Set), 1);
        assert_eq!(u16::from(Bit::Cleared), 0);
        assert_eq!(u32::from(Bit::Set), 1);
        assert_eq!(u32::from(Bit::Cleared), 0);
        assert_eq!(u64::from(Bit::Set), 1);
        assert_eq!(u64::from(Bit::Cleared), 0);
        assert_eq!(u128::from(Bit::Set), 1);
        assert_eq!(u128::from(Bit::Cleared), 0);

        assert_eq!(i8::from(Bit::Set), 1);
        assert_eq!(i8::from(Bit::Cleared), 0);
        assert_eq!(i16::from(Bit::Set), 1);
        assert_eq!(i16::from(Bit::Cleared), 0);
        assert_eq!(i32::from(Bit::Set), 1);
        assert_eq!(i32::from(Bit::Cleared), 0);
        assert_eq!(i64::from(Bit::Set), 1);
        assert_eq!(i64::from(Bit::Cleared), 0);
        assert_eq!(i128::from(Bit::Set), 1);
        assert_eq!(i128::from(Bit::Cleared), 0);
    }
}
