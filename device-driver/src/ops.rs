use core::ops::{BitOrAssign, Shl};

pub unsafe fn load_lsb0<T, ByteO: ByteOrder>(data: &[u8], start: usize, end: usize) -> T
where
    T: Default + From<u8> + Shl<usize, Output = T> + BitOrAssign,
{
    let mut output = T::default();

    let mut i = start;
    while i < end {
        let byte = ByteO::get_byte_from_index(data, i);

        if (i % 8 == 0) & (i + 8 <= end) {
            output |= T::from(byte) << (i - start);
            i += 8;
        } else {
            let bit = (byte >> i % 8) & 1;
            output |= T::from(bit) << (i - start);
            i += 1;
        }
    }

    output
}

pub unsafe fn load_msb0<T, ByteO: ByteOrder>(data: &[u8], start: usize, end: usize) -> T
where
    T: Default + From<u8> + Shl<usize, Output = T> + BitOrAssign,
{
    let mut output = T::default();

    let mut i = start;
    while i < end {
        let byte = ByteO::get_byte_from_index(data, i);

        if (i % 8 == 0) & (i + 8 <= end) {
            output |= T::from(byte) << (i - start);
            i += 8;
        } else {
            // Get the bit MSB0 (so reversed)
            let bit = (byte >> (7 - i % 8)) & 1;

            // We still need to store in normal order, so we need to reverse again, but within the index
            let (num_mirror_bits, pivot) = if i / 8 == start / 8 {
                // We are in the start byte
                let num_mirror_bits = (start + 1).next_multiple_of(8).min(end) - start;
                let pivot = start + num_mirror_bits / 2;

                (num_mirror_bits, pivot)
            } else {
                // We are in the end byte
                let num_mirror_bits = end - (end - 8).next_multiple_of(8);
                let pivot = end - ((num_mirror_bits + 1) / 2);

                (num_mirror_bits, pivot)
            };

            let mut diff = pivot as isize - i as isize;

            if diff <= 0 && num_mirror_bits % 2 == 0 {
                diff -= 1;
            }

            let j = i as isize + diff * 2 - ((num_mirror_bits % 2 == 0) as isize) * diff.signum();

            output |= T::from(bit) << (j as usize - start);
            i += 1;
        }
    }

    output
}

pub struct LE;
pub struct BE;

pub trait ByteOrder {
    type Opposite: ByteOrder;

    fn get_byte_index(data_len: usize, bit_index: usize) -> usize;
    unsafe fn get_byte_from_index(data: &[u8], bit_index: usize) -> u8 {
        *data.get_unchecked(Self::get_byte_index(data.len(), bit_index))
    }
}

impl ByteOrder for LE {
    type Opposite = BE;

    fn get_byte_index(_data_len: usize, bit_index: usize) -> usize {
        bit_index / 8
    }
}

impl ByteOrder for BE {
    type Opposite = LE;

    fn get_byte_index(data_len: usize, bit_index: usize) -> usize {
        data_len - (bit_index / 8) - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_u32_le_lsb0() {
        unsafe {
            assert_eq!(
                load_lsb0::<u32, LE>(&0b11111111111111111111000000u64.to_le_bytes(), 6, 26),
                0b11111111111111111111
            );
            assert_eq!(
                load_lsb0::<u32, LE>(&0xAABBCCDDEEu64.to_le_bytes(), 8, 32),
                0xBBCCDD,
            );
            assert_eq!(
                load_lsb0::<u32, LE>(&0xAABBCCDDEEu64.to_le_bytes(), 4, 28),
                0xBCCDDE,
            );
        }
    }

    #[test]
    fn test_load_u32_be_lsb0() {
        unsafe {
            assert_eq!(
                load_lsb0::<u32, BE>(&0b11111111111111111111000000u64.to_be_bytes(), 6, 26),
                0b11111111111111111111
            );
            assert_eq!(
                load_lsb0::<u32, BE>(&0xAABBCCDDEEu64.to_be_bytes(), 8, 32),
                0xBBCCDD,
            );
            assert_eq!(
                load_lsb0::<u32, BE>(&0xAABBCCDDEEu64.to_be_bytes(), 4, 28),
                0xBCCDDE,
            );
        }
    }

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
