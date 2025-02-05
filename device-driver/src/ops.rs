use core::ops::{BitOrAssign, Shl};

/// Load an integer from a byte slice between located at the `start`..`end` range.
/// The integer is loaded with the [LE] or [BE] byte order generic param and using lsb0 bit order.
///
/// ## Safety:
///
/// `start` and `end` must lie in the range `0..data.len()*8`.
pub unsafe fn load_lsb0<T, ByteO: ByteOrder>(data: &[u8], start: usize, end: usize) -> T
where
    T: Default + From<u8> + Shl<usize, Output = T> + BitOrAssign,
{
    // Start with 0
    let mut output = T::default();

    // Go through start..end, but in a while so we have more control over the index
    let mut i = start;
    while i < end {
        let byte = ByteO::get_byte_from_index(data, i);

        if (i % 8 == 0) & (i + 8 <= end) {
            // We are byte aligned and have a full byte of space left
            // Do a whole byte in one go for extra performance
            output |= T::from(byte) << (i - start);
            i += 8;
        } else {
            // Go bit by bit
            // Move the target bit all the way to the right so we know where it is
            let bit = (byte >> i % 8) & 1;
            // Shift the bit the proper amount to the left. The bit at `start` should be at index 0
            output |= T::from(bit) << (i - start);
            i += 1;
        }
    }

    output
}

/// Load an integer from a byte slice between located at the `start`..`end` range.
/// The integer is loaded with the [LE] or [BE] byte order generic param and using msb0 bit order.
/// This is more expensive than the [load_lsb0] function.
///
/// ## Safety:
///
/// `start` and `end` must lie in the range `0..data.len()*8`.
pub unsafe fn load_msb0<T, ByteO: ByteOrder>(data: &[u8], start: usize, end: usize) -> T
where
    T: Default + From<u8> + Shl<usize, Output = T> + BitOrAssign,
{
    // Start with 0
    let mut output = T::default();

    // Iterate over the bits, while retaining index control
    let mut i = start;
    while i < end {
        // Get the proper byte we should be looking at
        let byte = ByteO::get_byte_from_index(data, i);

        if (i % 8 == 0) & (i + 8 <= end) {
            // We are byte aligned and have a full byte of space left
            // Do a whole byte in one go for extra performance
            output |= T::from(byte) << (i - start);
            i += 8;
        } else {
            // Move bit by bit

            // We are doing msb0, so the indexing is reversed. However, we still need to store
            // the data in normal order. So we need to reverse the bits ourselves too.
            // We can't reverse the whole byte because then the bits end up in the wrong places.
            // So we reverse around a pivot and the pivot is in the middle of the bits that need
            // to be reversed.
            let (num_bits, pivot) = if i / 8 == start / 8 {
                // We are in the start byte

                // The number of bits we're dealing with is the difference between the start and the
                // first byte-aligned index or the end, whichever comes first
                let num_bits = (start + 1).next_multiple_of(8).min(end) - start;
                let pivot = start + num_bits / 2;

                (num_bits, pivot)
            } else {
                // We are in the end byte

                // The number of bits we're dealing with is the difference between the last aligned
                // bit index and the end
                let num_bits = end - (end - 8).next_multiple_of(8);
                // Calculate the pivot and force it to round down so any error is in the same direction
                // as in the start byte case
                let pivot = end - ((num_bits + 1) / 2);

                (num_bits, pivot)
            };
            let num_bits_even = num_bits % 2 == 0;

            // Calculate how far away our current bit is from the pivot.
            let mut diff = pivot as isize - i as isize;

            if diff <= 0 {
                // If we have an even number of bits, we should not have a diff of 0
                diff -= num_bits_even as isize;
            }

            // To do the reversing, we need to move the index towards the pivot
            // and then keep going the same distance again
            let mut j = i as isize + diff * 2;

            // Due to integer math, the pivot may not be exactly in the middle, so we need to correct for that.
            // The pivot is off by a half if we have an even number of bits.
            // But we've done a `* 2` previously now, so we can correct the half-error with a whole number.
            if diff > 0 {
                j -= 1 * num_bits_even as isize;
            } else {
                j += 1 * num_bits_even as isize;
            }

            // Get the bit MSB0 (so reversed)
            // Move the target bit all the way to the right so we know where it is
            let bit = (byte >> (7 - i % 8)) & 1;
            // Shift the bit the proper amount to the left. The bit at `start` should be at index 0
            output |= T::from(bit) << (j as usize - start);
            i += 1;
        }
    }

    output
}

/// Little endian byte order
pub struct LE;
/// Big endian byte order
pub struct BE;

/// Interface to byte order functions
pub trait ByteOrder {
    /// From the given bit index, get the byte index that is correct for this endianness
    fn get_byte_index(data_len: usize, bit_index: usize) -> usize;
    /// Get the byte from the data that is correct for the bit index and the endianness.
    ///
    /// ## Safety:
    ///
    /// `bit_index` must lie in the range `0..data.len()*8`.
    unsafe fn get_byte_from_index(data: &[u8], bit_index: usize) -> u8 {
        debug_assert!((0..data.len() * 8).contains(&bit_index));
        *data.get_unchecked(Self::get_byte_index(data.len(), bit_index))
    }
}

impl ByteOrder for LE {
    fn get_byte_index(_data_len: usize, bit_index: usize) -> usize {
        bit_index / 8
    }
}

impl ByteOrder for BE {
    fn get_byte_index(data_len: usize, bit_index: usize) -> usize {
        data_len - (bit_index / 8) - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Bytes<'a>(&'a [u8]);

    impl<'a> std::fmt::Binary for Bytes<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "[")?;
            for byte in self.0 {
                std::fmt::Binary::fmt(byte, f)?;
                write!(f, ",")?;
            }
            write!(f, "]")?;
            Ok(())
        }
    }

    #[test]
    fn same_as_bitvec() {
        use bitvec::{field::BitField, view::BitView};

        for _ in 0..1_000_000 {
            let mut data = vec![0u8; rand::random_range(1..=16)];
            rand::fill(&mut data[..]);
            let mut reversed_data = data.clone();
            reversed_data.reverse();

            let total_bits = data.len() * 8;

            let start = rand::random_range(0..total_bits - 1);
            let end = start + rand::random_range(1..=total_bits - start).min(32);

            println!("{start}..{end} @ {:#010b}", Bytes(&data));

            let test_value = unsafe { load_lsb0::<u32, LE>(&data, start, end) };
            let check_value = data.view_bits::<bitvec::order::Lsb0>()[start..end].load_le::<u32>();
            println!("LE Lsb0: {:016b} *", check_value);
            println!("LE Lsb0: {:016b}", test_value);
            assert_eq!(test_value, check_value);

            let test_value = unsafe { load_lsb0::<u32, BE>(&data, start, end) };
            let check_value =
                reversed_data.view_bits::<bitvec::order::Lsb0>()[start..end].load_le::<u32>();
            println!("BE Lsb0: {:016b} *", check_value);
            println!("BE Lsb0: {:016b}", test_value);
            assert_eq!(test_value, check_value);

            let test_value_le_msb0 = unsafe { load_msb0::<u32, LE>(&data, start, end) };
            let check_value_le_msb0 =
                data.view_bits::<bitvec::order::Msb0>()[start..end].load_le::<u32>();
            println!("LE Msb0: {:016b} *", check_value_le_msb0);
            println!("LE Msb0: {:016b}", test_value_le_msb0);

            let test_value_be_msb0 = unsafe { load_msb0::<u32, BE>(&data, start, end) };
            let check_value_be_msb0 =
                reversed_data.view_bits::<bitvec::order::Msb0>()[start..end].load_le::<u32>();
            println!("BE Msb0: {:016b} *", check_value_be_msb0);
            println!("BE Msb0: {:016b}", test_value_be_msb0);

            assert_eq!(test_value_le_msb0, check_value_le_msb0);
            assert_eq!(test_value_be_msb0, check_value_be_msb0);
        }
    }
}
