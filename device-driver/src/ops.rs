use core::ops::{BitAnd, BitOr, BitOrAssign, Shl, Shr};

/// Load an integer from a byte slice located at the `start`..`end` range.
/// The integer is loaded with the [LE] or [BE] byte order generic param and using lsb0 bit order.
///
/// ## Safety:
///
/// `start` and `end` must lie in the range `0..=data.len()*8` and the difference between them must be at least 1
#[inline(always)]
pub unsafe fn load_lsb0<T, ByteO: ByteOrder>(data: &[u8], start: usize, end: usize) -> T
where
    T: Default + Shl<usize, Output = T> + BitOrAssign + Integer + TruncateToU8,
{
    #[inline(never)]
    unsafe fn inner<T, ByteO: ByteOrder>(data: &[u8], start: usize, end: usize) -> T
    where
        T: Default + Shl<usize, Output = T> + BitOrAssign + TruncateToU8,
    {
        // Start with 0
        let mut output = T::default();

        // Go through start..end, but in a while so we have more control over the index
        let mut i = start;
        while i < end {
            let byte = unsafe { ByteO::get_byte_from_index(data, i) };

            if (i % 8 == 0) & (i + 8 <= end) {
                // We are byte aligned and have a full byte of space left
                // Do a whole byte in one go for extra performance
                output |= T::detruncate(byte) << (i - start);
                i += 8;
            } else {
                // Go bit by bit
                // Move the target bit all the way to the right so we know where it is
                let bit = (byte >> (i % 8)) & 1;
                // Shift the bit the proper amount to the left. The bit at `start` should be at index 0
                output |= T::detruncate(bit) << (i - start);
                i += 1;
            }
        }

        output
    }

    T::cast_deduplicate_back(unsafe { inner::<T::DedupType, ByteO>(data, start, end) })
        .sign_extend(unsafe { end.unchecked_sub(start).unchecked_sub(1) })
}

/// Store an integer into a byte slice located at the `start`..`end` range.
/// The integer is stored with the [LE] or [BE] byte order generic param and using lsb0 bit order.
///
/// ## Safety:
///
/// `start` and `end` must lie in the range `0..=data.len()*8` and the difference between them must be at least 1
#[inline(always)]
pub unsafe fn store_lsb0<T, ByteO: ByteOrder>(value: T, start: usize, end: usize, data: &mut [u8])
where
    T: Copy + TruncateToU8 + Shr<usize, Output = T> + Integer,
{
    #[inline(never)]
    unsafe fn inner<T, ByteO: ByteOrder>(value: T, start: usize, end: usize, data: &mut [u8])
    where
        T: Copy + TruncateToU8 + Shr<usize, Output = T>,
    {
        // Go through start..end, but in a while so we have more control over the index
        let mut i = start;
        while i < end {
            let byte = unsafe { ByteO::get_byte_from_index_mut(data, i) };

            if (i % 8 == 0) & (i + 8 <= end) {
                // We are byte aligned and have a full byte of space left
                // Do a whole byte in one go for extra performance
                *byte = (value >> (i - start)).truncate();
                i += 8;
            } else {
                // Go bit by bit
                // Move the target bit all the way to the right so we know where it is
                let bit = (value >> (i - start)).truncate() & 1;

                // Clear the bit
                *byte &= !(1 << (i % 8));
                // If the bit is set, set the bit in the byte
                // Not if statement here since this is faster and smaller
                *byte |= bit << (i % 8);

                i += 1;
            }
        }
    }

    unsafe { inner::<T::DedupType, ByteO>(value.cast_deduplicate(), start, end, data) }
}

/// Load an integer from a byte slice located at the `start`..`end` range.
/// The integer is loaded with the [LE] or [BE] byte order generic param and using msb0 bit order.
/// This is more expensive than the [load_lsb0] function.
///
/// ## Safety:
///
/// `start` and `end` must lie in the range `0..=data.len()*8` and the difference between them must be at least 1
#[inline(always)]
pub unsafe fn load_msb0<T, ByteO: ByteOrder>(data: &[u8], start: usize, end: usize) -> T
where
    T: Default + Shl<usize, Output = T> + BitOrAssign + Integer + TruncateToU8,
{
    #[inline(never)]
    unsafe fn inner<T, ByteO: ByteOrder>(data: &[u8], start: usize, end: usize) -> T
    where
        T: Default + Shl<usize, Output = T> + BitOrAssign + TruncateToU8,
    {
        // Start with 0
        let mut output = T::default();

        // Iterate over the bits, while retaining index control
        let mut i = start;
        while i < end {
            // Get the proper byte we should be looking at
            let byte = unsafe { ByteO::get_byte_from_index(data, i) };

            if (i % 8 == 0) & (i + 8 <= end) {
                // We are byte aligned and have a full byte of space left
                // Do a whole byte in one go for extra performance
                output |= T::detruncate(byte) << (i - start);
                i += 8;
            } else {
                // Move bit by bit

                let j = pivot_msb0(start, end, i);

                // Get the bit MSB0 (so reversed)
                // Move the target bit all the way to the right so we know where it is
                let bit = (byte >> (7 - i % 8)) & 1;
                // Shift the bit the proper amount to the left. The bit at `start` should be at index 0
                output |= T::detruncate(bit) << (j - start);
                i += 1;
            }
        }

        output
    }

    T::cast_deduplicate_back(unsafe { inner::<T::DedupType, ByteO>(data, start, end) })
        .sign_extend(unsafe { end.unchecked_sub(start).unchecked_sub(1) })
}

/// Store an integer into byte slice located at the `start`..`end` range.
/// The integer is stored with the [LE] or [BE] byte order generic param and using msb0 bit order.
/// This is more expensive than the [store_lsb0] function.
///
/// ## Safety:
///
/// `start` and `end` must lie in the range `0..=data.len()*8` and the difference between them must be at least 1
#[inline(always)]
pub unsafe fn store_msb0<T, ByteO: ByteOrder>(value: T, start: usize, end: usize, data: &mut [u8])
where
    T: Copy + TruncateToU8 + Shr<usize, Output = T> + Integer,
{
    #[inline(never)]
    unsafe fn inner<T, ByteO: ByteOrder>(value: T, start: usize, end: usize, data: &mut [u8])
    where
        T: Copy + TruncateToU8 + Shr<usize, Output = T>,
    {
        // Iterate over the bits, while retaining index control
        let mut i = start;
        while i < end {
            // Get the proper byte we should be looking at
            let byte = unsafe { ByteO::get_byte_from_index_mut(data, i) };

            if (i % 8 == 0) & (i + 8 <= end) {
                // We are byte aligned and have a full byte of space left
                // Do a whole byte in one go for extra performance
                *byte = (value >> (i - start)).truncate();
                i += 8;
            } else {
                // Move bit by bit

                let j = pivot_msb0(start, end, i);

                // Get the bit MSB0 (so reversed)
                // Move the target bit all the way to the right so we know where it is
                let bit = (value >> (j - start)).truncate() & 1;

                // Clear the bit
                *byte &= !(1 << (7 - i % 8));
                // If the bit is set, set the bit in the byte
                // Not if statement here since this is faster and smaller
                *byte |= bit << (7 - i % 8);

                i += 1;
            }
        }
    }

    unsafe { inner::<T::DedupType, ByteO>(value.cast_deduplicate(), start, end, data) }
}

#[inline]
fn pivot_msb0(start: usize, end: usize, i: usize) -> usize {
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
        let pivot = end - num_bits.div_ceil(2);

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
        j -= num_bits_even as isize;
    } else {
        j += num_bits_even as isize;
    }

    j as usize
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
        unsafe { *data.get_unchecked(Self::get_byte_index(data.len(), bit_index)) }
    }

    /// Get a mutable reference to the byte from the data that is correct for the bit index and the endianness.
    ///
    /// ## Safety:
    ///
    /// `bit_index` must lie in the range `0..data.len()*8`.
    unsafe fn get_byte_from_index_mut(data: &mut [u8], bit_index: usize) -> &mut u8 {
        debug_assert!((0..data.len() * 8).contains(&bit_index));
        unsafe { data.get_unchecked_mut(Self::get_byte_index(data.len(), bit_index)) }
    }
}

impl ByteOrder for LE {
    #[inline]
    fn get_byte_index(_data_len: usize, bit_index: usize) -> usize {
        bit_index / 8
    }
}

impl ByteOrder for BE {
    #[inline]
    fn get_byte_index(data_len: usize, bit_index: usize) -> usize {
        data_len - (bit_index / 8) - 1
    }
}

pub trait TruncateToU8 {
    fn truncate(self) -> u8;
    fn detruncate(val: u8) -> Self;
}

macro_rules! impl_truncate_to_u8 {
    ($($target:ty),*) => {
        $(
            impl TruncateToU8 for $target {
                fn truncate(self) -> u8 {
                    self as u8
                }
                fn detruncate(val: u8) -> Self {
                    val as Self
                }
            }
        )*
    };
}

impl_truncate_to_u8!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

pub trait Integer:
    Sized + Copy + Shl<usize, Output = Self> + BitOr<Output = Self> + BitAnd<Output = Self> + PartialEq
{
    type DedupType: Default
        + From<u8>
        + Shl<usize, Output = Self::DedupType>
        + BitOrAssign
        + Copy
        + TruncateToU8
        + Shr<usize, Output = Self::DedupType>;

    const SIGN_EXTEND_ONES: Self;
    const ZERO: Self;
    const ONE: Self;
    const BITS: usize;

    /// Cast the integer to a common type to avoid generics bloat
    fn cast_deduplicate(self) -> Self::DedupType;
    fn cast_deduplicate_back(val: Self::DedupType) -> Self;

    #[inline]
    fn sign_extend(self, sign_bit_index: usize) -> Self {
        if Self::SIGN_EXTEND_ONES == Self::ZERO {
            return self;
        }

        if sign_bit_index > Self::BITS {
            return self;
        }

        let sign_bit = Self::ONE << sign_bit_index;

        if (self & sign_bit) != sign_bit {
            return self;
        }

        self | (Self::SIGN_EXTEND_ONES << sign_bit_index)
    }
}

macro_rules! impl_integer {
    ($target:ty, $dedup:ty, $sign_ones:expr) => {
        impl Integer for $target {
            type DedupType = $dedup;
            const SIGN_EXTEND_ONES: Self = $sign_ones;
            const ZERO: Self = 0;
            const ONE: Self = 1;
            const BITS: usize = Self::BITS as usize;

            fn cast_deduplicate(self) -> Self::DedupType {
                self as _
            }

            fn cast_deduplicate_back(val: Self::DedupType) -> Self {
                val as _
            }
        }
    };
    ($target:ty, $dedup:ty, $sign_ones:expr, $cfg:meta) => {
        #[$cfg]
        impl_integer!($target, $dedup, $sign_ones);
    };
}

impl_integer!(u8, usize, 0);
impl_integer!(u16, usize, 0);
impl_integer!(u32, u32, 0, cfg(target_pointer_width = "16"));
impl_integer!(u32, usize, 0, cfg(not(target_pointer_width = "16")));
impl_integer!(u64, u64, 0, cfg(target_pointer_width = "16"));
impl_integer!(u64, u64, 0, cfg(target_pointer_width = "32"));
impl_integer!(u64, usize, 0, cfg(target_pointer_width = "64"));
impl_integer!(u128, u128, 0);
impl_integer!(usize, usize, 0);
impl_integer!(i8, usize, !0);
impl_integer!(i16, usize, !0);
impl_integer!(i32, u32, !0, cfg(target_pointer_width = "16"));
impl_integer!(i32, usize, !0, cfg(not(target_pointer_width = "16")));
impl_integer!(i64, u64, !0, cfg(target_pointer_width = "16"));
impl_integer!(i64, u64, !0, cfg(target_pointer_width = "32"));
impl_integer!(i64, usize, !0, cfg(target_pointer_width = "64"));
impl_integer!(i128, u128, !0);
impl_integer!(isize, usize, !0);

#[cfg(test)]
mod tests {
    use super::*;

    struct Bytes<'a>(&'a [u8]);

    impl std::fmt::Binary for Bytes<'_> {
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
    fn load_same_as_bitvec() {
        use bitvec::{field::BitField, view::BitView};

        for _ in 0..10_000 {
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

            let test_value = unsafe { load_msb0::<u32, LE>(&data, start, end) };
            let check_value = data.view_bits::<bitvec::order::Msb0>()[start..end].load_le::<u32>();
            println!("LE Msb0: {:016b} *", check_value);
            println!("LE Msb0: {:016b}", test_value);
            assert_eq!(test_value, check_value);

            let test_value = unsafe { load_msb0::<u32, BE>(&data, start, end) };
            let check_value =
                reversed_data.view_bits::<bitvec::order::Msb0>()[start..end].load_le::<u32>();
            println!("BE Msb0: {:016b} *", check_value);
            println!("BE Msb0: {:016b}", test_value);
            assert_eq!(test_value, check_value);
        }
    }

    #[test]
    fn store_same_as_bitvec() {
        use bitvec::{field::BitField, view::BitView};

        for _ in 0..10_000 {
            let mut data = vec![0u8; rand::random_range(1..=16)];
            rand::fill(&mut data[..]);
            let mut reversed_data = data.clone();
            reversed_data.reverse();

            let total_bits = data.len() * 8;
            let start = rand::random_range(0..total_bits - 1);
            let end = start + rand::random_range(1..=total_bits - start).min(32);

            let input_data = rand::random::<u32>();
            println!(
                "{input_data:#034b} -> {start}..{end} @ {:#010b}",
                Bytes(&data)
            );

            let mut test_data = data.clone();
            unsafe { store_lsb0::<_, LE>(input_data, start, end, &mut test_data) };
            let mut check_data = data.clone();
            check_data.view_bits_mut::<bitvec::order::Lsb0>()[start..end].store_le(input_data);
            println!("LE Lsb0: {:#010b} *", Bytes(&check_data));
            println!("LE Lsb0: {:#010b}", Bytes(&test_data));
            assert_eq!(test_data, check_data);

            let mut test_data = data.clone();
            unsafe { store_lsb0::<_, BE>(input_data, start, end, &mut test_data) };
            let mut check_data = reversed_data.clone();
            check_data.view_bits_mut::<bitvec::order::Lsb0>()[start..end].store_le(input_data);
            check_data.reverse();
            println!("BE Lsb0: {:#010b} *", Bytes(&check_data));
            println!("BE Lsb0: {:#010b}", Bytes(&test_data));
            assert_eq!(test_data, check_data);

            let mut test_data = data.clone();
            unsafe { store_msb0::<_, LE>(input_data, start, end, &mut test_data) };
            let mut check_data = data.clone();
            check_data.view_bits_mut::<bitvec::order::Msb0>()[start..end].store_le(input_data);
            println!("LE Msb0: {:#010b} *", Bytes(&check_data));
            println!("LE Msb0: {:#010b}", Bytes(&test_data));

            let mut test_data1 = data.clone();
            unsafe { store_msb0::<_, BE>(input_data, start, end, &mut test_data1) };
            let mut check_data1 = reversed_data.clone();
            check_data1.view_bits_mut::<bitvec::order::Msb0>()[start..end].store_le(input_data);
            check_data1.reverse();
            println!("BE Msb0: {:#010b} *", Bytes(&check_data1));
            println!("BE Msb0: {:#010b}", Bytes(&test_data1));
            assert_eq!(test_data, check_data);
            assert_eq!(test_data1, check_data1);
        }
    }

    #[test]
    fn twos_complement() {
        for i in 1..=32 {
            println!("Bit width: {i}");
            let mut data = [0; 4];

            unsafe { store_lsb0::<i32, LE>(-1, 0, i, &mut data) };
            let read_back = unsafe { load_lsb0::<i32, LE>(&data, 0, i) };

            assert_eq!(read_back, -1);
        }
    }
}
