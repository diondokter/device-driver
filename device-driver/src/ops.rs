use core::ops::{BitAnd, BitOrAssign, Shl, Shr};

pub unsafe fn load_lsb0<T, ByteO: ByteOrder>(data: &[u8], start: usize, end: usize) -> T
where
    T: Default
        + From<u8>
        + Shl<usize, Output = T>
        + BitOrAssign,
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
    T: Default
        + From<u8>
        + Shr<usize, Output = T>
        + BitAnd<T, Output = T>
        + Shl<usize, Output = T>
        + BitOrAssign
        + SwapBytes,
{
    let new_start = data.len() * 8 - end;
    let new_end = data.len() * 8 - start;
    load_lsb0::<T, ByteO::Opposite>(data, new_start, new_end)//.swap_bytes()
}

pub struct LE;
pub struct BE;

pub trait ByteOrder {
    type Opposite: ByteOrder;
    unsafe fn get_byte_from_index(data: &[u8], bit_index: usize) -> u8;
}

impl ByteOrder for LE {
    type Opposite = BE;

    unsafe fn get_byte_from_index(data: &[u8], bit_index: usize) -> u8 {
        *data.get_unchecked(bit_index / 8)
    }
}

impl ByteOrder for BE {
    type Opposite = LE;

    unsafe fn get_byte_from_index(data: &[u8], bit_index: usize) -> u8 {
        *data.get_unchecked(data.len() - (bit_index / 8) - 1)
    }
}

pub trait SwapBytes {
    fn swap_bytes(self) -> Self;
}

macro_rules! impl_swap_bytes {
    ($integer:ty) => {
        impl SwapBytes for $integer {
            fn swap_bytes(self) -> Self {
                <$integer>::swap_bytes(self)
            }
        }
    };
}

impl_swap_bytes!(u8);
impl_swap_bytes!(u16);
impl_swap_bytes!(u32);
impl_swap_bytes!(u64);
impl_swap_bytes!(u128);

impl_swap_bytes!(i8);
impl_swap_bytes!(i16);
impl_swap_bytes!(i32);
impl_swap_bytes!(i64);
impl_swap_bytes!(i128);

#[cfg(test)]
mod tests {
    use bitvec::view::BitView;

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

        for _ in 0..1_000 {
            let mut data = vec![0u8; rand::random_range(2..=2)];
            rand::fill(&mut data[..]);
            let mut reversed_data = data.clone();
            reversed_data.reverse();

            let total_bits = data.len() * 8;

            let start = rand::random_range(0..total_bits - 1);
            let end = start + rand::random_range(1..=total_bits - start).min(32);

            println!("{start}..{end} @ {:#010b}", Bytes(&data));

            // let test_value = unsafe { load_lsb0::<u32, LE>(&data, start, end) };
            // let check_value = data.view_bits::<bitvec::order::Lsb0>()[start..end].load_le::<u32>();
            // println!("LE Lsb0: {:08b} *", Bytes(&check_value.to_be_bytes()));
            // println!("LE Lsb0: {:08b}", Bytes(&test_value.to_be_bytes()));
            // assert_eq!(test_value, check_value);

            // let test_value = unsafe { load_lsb0::<u32, BE>(&data, start, end) };
            // let check_value =
            //     reversed_data.view_bits::<bitvec::order::Lsb0>()[start..end].load_le::<u32>();
            // println!("BE Lsb0: {:08b} *", Bytes(&check_value.to_be_bytes()));
            // println!("BE Lsb0: {:08b}", Bytes(&test_value.to_be_bytes()));
            // assert_eq!(test_value, check_value);

            let test_value_le_msb0 = unsafe { load_msb0::<u32, LE>(&data, start, end) };
            let check_value_le_msb0 = data.view_bits::<bitvec::order::Msb0>()[start..end].load_le::<u32>();
            println!("LE Msb0: {:08b} *", Bytes(&check_value_le_msb0.to_be_bytes()));
            println!("LE Msb0: {:08b}", Bytes(&test_value_le_msb0.to_be_bytes()));

            let test_value_be_msb0 = unsafe { load_msb0::<u32, BE>(&data, start, end) };
            let check_value_be_msb0 =
                reversed_data.view_bits::<bitvec::order::Msb0>()[start..end].load_le::<u32>();
            println!(
                "BE Msb0: {:08b} *",
                Bytes(&check_value_be_msb0.to_be_bytes())
            );
            println!("BE Msb0: {:08b}", Bytes(&test_value_be_msb0.to_be_bytes()));

            assert_eq!(test_value_le_msb0, check_value_le_msb0);
            assert_eq!(test_value_be_msb0, check_value_be_msb0);
        }
    }
}
