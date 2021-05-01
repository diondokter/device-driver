//! Module containing utilities. As a user of this crate, you shouldn't have to use this directly and
//! because it is used in the macros, these need to be public.
//!

use core::fmt::{Debug, Formatter, Result};

/// A wrapper around a slice that formats everything to upper hex
pub struct SliceHexFormatter<'a> {
    slice: &'a [u8],
}

impl<'a> SliceHexFormatter<'a> {
    /// Wrap around the value
    pub fn new(slice: &'a [u8]) -> Self {
        Self { slice }
    }
}

impl<'a> Debug for SliceHexFormatter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "0x")?;
        for elem in self.slice {
            write!(f, "{:02X}", elem)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_hex_formatter() {
        assert_eq!(
            format!(
                "{:?}",
                SliceHexFormatter::new(&[0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01])
            ),
            "DEADBEEF0001".to_string()
        );
    }
}
