#![allow(async_fn_in_trait)]
#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]
#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

use core::fmt::{Debug, Display};
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

pub use embedded_io;
pub use embedded_io_async;

mod register;
pub use register::*;
mod command;
pub use command::*;
mod buffer;
pub use buffer::*;

#[doc(hidden)]
pub mod ops;

#[cfg(feature = "macros")]
pub use device_driver_macros::*;

#[doc(hidden)]
/// # Safety
///
/// Must only be implemented on types that are align(1) (so they introduce no padding bytes).
/// This is used to cast the fieldset to a slice
pub unsafe trait FieldSet: Default + Copy {
    type Unpacked: UnpackedFieldSet<Packed = Self>;

    /// The size of the field set in number of bits
    const SIZE_BITS: u32;

    fn get_inner_buffer(&self) -> &[u8];
    fn get_inner_buffer_mut(&mut self) -> &mut [u8];

    fn unpack(self) -> Self::Unpacked;
}

#[doc(hidden)]
pub trait UnpackedFieldSet {
    type Packed: FieldSet<Unpacked = Self>;

    fn pack(self) -> Self::Packed;
}

/// Type state value for a packed [FieldSetArray]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Packed;
/// Type state value for an unpacked [FieldSetArray]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Unpacked;

/// An array of field sets.
///
/// Can be packed and unpacked
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct FieldSetArray<T: FieldSet, const N: usize, P> {
    sets: [T; N],
    _phantom: PhantomData<P>,
}

impl<T: FieldSet, const N: usize> FieldSetArray<T, N, Unpacked> {
    /// Create a new array from the given initial value. All elements of the array will contain the value
    pub const fn new_with(val: T) -> Self {
        Self {
            sets: [val; N],
            _phantom: PhantomData,
        }
    }

    /// Pack the bits of the array together. After doing this, the type is a valid [FieldSet].
    pub fn pack(self) -> FieldSetArray<T, N, Packed> {
        if T::SIZE_BITS.is_multiple_of(8) {
            FieldSetArray {
                sets: self.sets,
                _phantom: PhantomData,
            }
        } else {
            todo!("Support non-multiple-of-8 registers in FieldSetArray");
        }
    }
}

impl<T: FieldSet, const N: usize, I> Index<I> for FieldSetArray<T, N, Unpacked>
where
    [T]: Index<I>,
{
    type Output = <[T] as Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        Index::index(self.sets.as_slice(), index)
    }
}

impl<T: FieldSet, const N: usize, I> IndexMut<I> for FieldSetArray<T, N, Unpacked>
where
    [T]: IndexMut<I>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(self.sets.as_mut_slice(), index)
    }
}

impl<T: FieldSet, const N: usize> From<FieldSetArray<T, N, Unpacked>> for [T; N] {
    fn from(value: FieldSetArray<T, N, Unpacked>) -> Self {
        value.sets
    }
}

impl<T: FieldSet, const N: usize> From<[T; N]> for FieldSetArray<T, N, Unpacked> {
    fn from(value: [T; N]) -> Self {
        Self {
            sets: value,
            _phantom: PhantomData,
        }
    }
}

impl<T: FieldSet, const N: usize> FieldSetArray<T, N, Packed> {
    /// Unpack the fieldsets so they all contain their own data again. After this, it's a valid `[T;N]`.
    pub fn unpack(self) -> FieldSetArray<T, N, Unpacked> {
        if T::SIZE_BITS.is_multiple_of(8) {
            FieldSetArray {
                sets: self.sets,
                _phantom: PhantomData,
            }
        } else {
            todo!("Support non-multiple-of-8 registers in FieldSetArray");
        }
    }
}

impl<const N: usize, T: FieldSet> UnpackedFieldSet for FieldSetArray<T, N, Unpacked> {
    type Packed = FieldSetArray<T, N, Packed>;

    fn pack(self) -> Self::Packed {
        self.pack()
    }
}

unsafe impl<const N: usize, T: FieldSet> FieldSet for FieldSetArray<T, N, Packed> {
    type Unpacked = FieldSetArray<T, N, Unpacked>;

    const SIZE_BITS: u32 = T::SIZE_BITS * N as u32;

    fn get_inner_buffer(&self) -> &[u8] {
        let ptr = self.sets.as_ptr();
        // Safety: This is safe because FieldSets are align 1 and `SIZE_BITS` is always smaller than the total allocated bits
        unsafe {
            core::slice::from_raw_parts(ptr.cast::<u8>(), Self::SIZE_BITS.div_ceil(8) as usize)
        }
    }

    fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
        let ptr = self.sets.as_mut_ptr();
        // Safety: This is safe because FieldSets are align 1 and `SIZE_BITS` is always smaller than the total allocated bits
        unsafe {
            core::slice::from_raw_parts_mut(ptr.cast::<u8>(), Self::SIZE_BITS.div_ceil(8) as usize)
        }
    }

    fn unpack(self) -> Self::Unpacked {
        self.unpack()
    }
}

impl<const N: usize, T: FieldSet, P> Default for FieldSetArray<T, N, P> {
    fn default() -> Self {
        Self {
            sets: [T::default(); N],
            _phantom: PhantomData,
        }
    }
}

impl<const N: usize, T: FieldSet + Debug> Debug for FieldSetArray<T, N, Unpacked> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.sets.iter()).finish()
    }
}

#[cfg(feature = "defmt")]
impl<const N: usize, T: FieldSet + defmt::Format> defmt::Format for FieldSetArray<T, N, Unpacked> {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "[");

        for (i, fs) in self.sets.iter().enumerate() {
            if i == 0 {
                defmt::write!(fmt, "{}", fs);
            } else {
                defmt::write!(fmt, ", {}", fs);
            }
        }

        defmt::write!(fmt, "]");
    }
}

/// Trait implemented on every generated block/device.
pub trait Block {
    /// The interface used by the block
    type Interface;
    /// The register address type
    type RegisterAddressType;
    /// The command address type
    type CommandAddressType;
    /// The buffer address type
    type BufferAddressType;

    /// Get a reference to the inner interface.
    /// With it you can do out-of-band operations that aren't defined in the generated code.
    fn interface(&mut self) -> &mut Self::Interface;

    /// Start a multi-read transaction
    ///
    /// You can chain reads by calling [register::MultiRegisterOperation::with].
    /// Once chained, call [register::MultiRegisterOperation::execute] to perform the read.
    fn multi_read(
        &mut self,
    ) -> register::MultiRegisterOperation<'_, Self, Self::RegisterAddressType, (), RO>
    where
        Self: Sized,
    {
        register::MultiRegisterOperation {
            device: self,
            start_address: None,
            field_sets: (),
            bit_sum: 0,
            _phantom: PhantomData,
        }
    }

    /// Start a multi-write transaction
    ///
    /// You can chain writes by calling [register::MultiRegisterOperation::with].
    /// Once chained, call [register::MultiRegisterOperation::execute] to perform the read.
    fn multi_write(
        &mut self,
    ) -> register::MultiRegisterOperation<'_, Self, Self::RegisterAddressType, (), WO>
    where
        Self: Sized,
    {
        register::MultiRegisterOperation {
            device: self,
            start_address: None,
            field_sets: (),
            bit_sum: 0,
            _phantom: PhantomData,
        }
    }

    /// Start a multi-modify transaction
    ///
    /// You can chain modifies by calling [register::MultiRegisterOperation::with].
    /// Once chained, call [register::MultiRegisterOperation::execute] to perform the read.
    fn multi_modify(
        &mut self,
    ) -> register::MultiRegisterOperation<'_, Self, Self::RegisterAddressType, (), RW>
    where
        Self: Sized,
    {
        register::MultiRegisterOperation {
            device: self,
            start_address: None,
            field_sets: (),
            bit_sum: 0,
            _phantom: PhantomData,
        }
    }
}

/// The error returned by the generated [`TryFrom`]s.
/// It contains the base type of the enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ConversionError<T> {
    /// The value of the thing that was tried to be converted
    pub source: T,
    /// The name of the target type
    pub target: &'static str,
}

impl<T: Display> Display for ConversionError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Could not convert value from `{}` to type `{}`",
            self.source, self.target
        )
    }
}

impl<T: Display + Debug> core::error::Error for ConversionError<T> {}

#[doc(hidden)]
pub struct WO;
#[doc(hidden)]
pub struct RO;
#[doc(hidden)]
pub struct RW;

#[doc(hidden)]
pub trait ReadCapability {}
#[doc(hidden)]
pub trait WriteCapability {}

impl WriteCapability for WO {}

impl ReadCapability for RO {}

impl WriteCapability for RW {}
impl ReadCapability for RW {}

#[doc(hidden)]
pub trait UnpackedFsSet: Sized {
    type Value;
    type ValueMut<'a>
    where
        Self: 'a;
    type Next<T: UnpackedFieldSet>;
    type Packed: PackedFsSet<Unpacked = Self>;

    fn push<T: UnpackedFieldSet>(self, val: T) -> Self::Next<T>;
    fn to_value(self) -> Self::Value;
    fn as_value_mut(&mut self) -> Self::ValueMut<'_>;

    fn pack(self) -> Self::Packed;
}

/// # Safety
///
/// May only be implemented on type that you can safely implement [UnpackedFsSet::as_slice_mut] for
#[doc(hidden)]
pub unsafe trait PackedFsSet: Sized {
    type Unpacked: UnpackedFsSet<Packed = Self>;

    fn as_slice_mut(&mut self) -> &mut [u8] {
        // Safety: Trait is only implemented on types that can do this.
        unsafe {
            let len = core::mem::size_of::<Self>();
            let ptr = self as *mut Self;
            core::slice::from_raw_parts_mut(ptr.cast(), len)
        }
    }

    fn unpack(self) -> Self::Unpacked;
}

impl UnpackedFsSet for () {
    type Value = ();
    type ValueMut<'a> = &'a mut ();
    type Next<T: UnpackedFieldSet> = T;
    type Packed = ();

    fn push<T: UnpackedFieldSet>(self, val: T) -> Self::Next<T> {
        val
    }

    fn to_value(self) -> Self::Value {}

    fn as_value_mut(&mut self) -> Self::ValueMut<'_> {
        self
    }

    fn pack(self) -> Self::Packed {}
}

unsafe impl PackedFsSet for () {
    type Unpacked = ();

    fn unpack(self) -> Self::Unpacked {}
}

impl<A: UnpackedFieldSet> UnpackedFsSet for A {
    type Value = A;
    type ValueMut<'a>
        = &'a mut A
    where
        A: 'a;
    type Next<T: UnpackedFieldSet> = Fs2<A, T>;
    type Packed = A::Packed;

    fn push<T: UnpackedFieldSet>(self, val: T) -> Self::Next<T> {
        Fs2(self, val)
    }

    fn to_value(self) -> Self::Value {
        self
    }

    fn as_value_mut(&mut self) -> Self::ValueMut<'_> {
        self
    }

    fn pack(self) -> Self::Packed {
        <A as UnpackedFieldSet>::pack(self)
    }
}

unsafe impl<A: FieldSet> PackedFsSet for A {
    type Unpacked = A::Unpacked;

    fn unpack(self) -> Self::Unpacked {
        <A as FieldSet>::unpack(self)
    }
}

macro_rules! create_fs {
    ($name:ident -> $name_next:ident, $(($tname:ident: $tnum:tt)),+) => {
        /// Combined fieldsets
        #[derive(Debug)]
        #[repr(C)]
        #[doc(hidden)]
        pub struct $name<$($tname),*>($($tname),*);

        impl<$($tname: UnpackedFieldSet),*> UnpackedFsSet for $name<$($tname),*> {
            type Value = ($($tname),*);
            type ValueMut<'a>
                = ($(&'a mut $tname),*)
            where
                $($tname: 'a),*;
            type Next<Next: UnpackedFieldSet> = $name_next<$($tname),*, Next>;
            type Packed = $name<$($tname::Packed),*>;

            fn push<Next: UnpackedFieldSet>(self, val: Next) -> Self::Next<Next> {
                $name_next($(self.$tnum),*, val)
            }

            fn to_value(self) -> Self::Value {
                ($(self.$tnum),*)
            }

            fn as_value_mut(&mut self) -> Self::ValueMut<'_> {
                ($(&mut self.$tnum),*)
            }

            fn pack(self) -> Self::Packed {
                $name($(self.$tnum.pack()),*)
            }
        }

        unsafe impl<$($tname: FieldSet),*> PackedFsSet for $name<$($tname),*> {
            type Unpacked = $name<$($tname::Unpacked),*>;

            fn unpack(self) -> Self::Unpacked {
                $name($(self.$tnum.unpack()),*)
            }
        }
    };
    ($name:ident -> !, $(($tname:ident: $tnum:tt)),+) => {
        /// Combined fieldsets
        #[derive(Debug)]
        #[repr(C)]
        #[doc(hidden)]
        pub struct $name<$($tname),*>($($tname),*);

        impl<$($tname: UnpackedFieldSet),*> UnpackedFsSet for $name<$($tname),*> {
            type Value = ($($tname),*);
            type ValueMut<'a>
                = ($(&'a mut $tname),*)
            where
                $($tname: 'a),*;
            type Next<Next: UnpackedFieldSet> = core::convert::Infallible;
            type Packed = $name<$($tname::Packed),*>;

            fn push<Next: UnpackedFieldSet>(self, _val: Next) -> Self::Next<Next> {
                panic!()
            }

            fn to_value(self) -> Self::Value {
                ($(self.$tnum),*)
            }

            fn as_value_mut(&mut self) -> Self::ValueMut<'_> {
                ($(&mut self.$tnum),*)
            }

            fn pack(self) -> Self::Packed {
                $name($(self.$tnum.pack()),*)
            }
        }

        unsafe impl<$($tname: FieldSet),*> PackedFsSet for $name<$($tname),*> {
            type Unpacked = $name<$($tname::Unpacked),*>;

            fn unpack(self) -> Self::Unpacked {
                $name($(self.$tnum.unpack()),*)
            }
        }
    };
}

create_fs!(Fs2  ->  Fs3, (A: 0), (B: 1));
create_fs!(Fs3  ->  Fs4, (A: 0), (B: 1), (C: 2));
create_fs!(Fs4  ->  Fs5, (A: 0), (B: 1), (C: 2), (D: 3));
create_fs!(Fs5  ->  Fs6, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4));
create_fs!(Fs6  ->  Fs7, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5));
create_fs!(Fs7  ->  Fs8, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6));
create_fs!(Fs8  ->  Fs9, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7));
create_fs!(Fs9  -> Fs10, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8));
create_fs!(Fs10 -> Fs11, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9));
create_fs!(Fs11 -> Fs12, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10));
create_fs!(Fs12 -> Fs13, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11));
create_fs!(Fs13 -> Fs14, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12));
create_fs!(Fs14 -> Fs15, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13));
create_fs!(Fs15 -> Fs16, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14));
create_fs!(Fs16 -> Fs17, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14), (P: 15));
create_fs!(Fs17 -> Fs18, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14), (P: 15), (Q: 16));
create_fs!(Fs18 -> Fs19, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14), (P: 15), (Q: 16), (R: 17));
create_fs!(Fs19 -> Fs20, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14), (P: 15), (Q: 16), (R: 17), (S: 18));
create_fs!(Fs20 -> Fs21, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14), (P: 15), (Q: 16), (R: 17), (S: 18), (T: 19));
create_fs!(Fs21 -> Fs22, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14), (P: 15), (Q: 16), (R: 17), (S: 18), (T: 19), (U: 20));
create_fs!(Fs22 -> Fs23, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14), (P: 15), (Q: 16), (R: 17), (S: 18), (T: 19), (U: 20), (V: 21));
create_fs!(Fs23 -> Fs24, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14), (P: 15), (Q: 16), (R: 17), (S: 18), (T: 19), (U: 20), (V: 21), (W: 22));
create_fs!(Fs24 -> Fs25, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14), (P: 15), (Q: 16), (R: 17), (S: 18), (T: 19), (U: 20), (V: 21), (W: 22), (X: 23));
create_fs!(Fs25 -> Fs26, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14), (P: 15), (Q: 16), (R: 17), (S: 18), (T: 19), (U: 20), (V: 21), (W: 22), (X: 23), (Y: 24));
create_fs!(Fs26 ->    !, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14), (P: 15), (Q: 16), (R: 17), (S: 18), (T: 19), (U: 20), (V: 21), (W: 22), (X: 23), (Y: 24), (Z: 25));

#[doc(hidden)]
pub trait Repeating {
    type Index: Clone;

    /// Calculate an address with the index
    #[allow(private_bounds)]
    fn calc_address<AddressType: Address>(start: AddressType, index: Self::Index) -> AddressType;
}
#[doc(hidden)]
pub trait NotRepeating {}
impl NotRepeating for () {}
#[doc(hidden)]
pub trait LinearRepeating: Repeating {
    const COUNT: u16;
    const STRIDE: i32;

    fn assert_len_and_index(len: usize, index: Self::Index);
}

#[doc(hidden)]
pub struct ArrayRepeat<const COUNT: u16, const STRIDE: i32>;
impl<const COUNT: u16, const STRIDE: i32> Repeating for ArrayRepeat<COUNT, STRIDE> {
    type Index = usize;

    #[track_caller]
    #[inline(always)]
    fn calc_address<AddressType: Address>(start: AddressType, index: Self::Index) -> AddressType {
        assert!(
            index < COUNT as usize,
            "Index out of range: {index} (array len: {COUNT})"
        );
        let offset = index as i32 * STRIDE;
        start.add(offset)
    }
}
impl<const COUNT: u16, const STRIDE: i32> LinearRepeating for ArrayRepeat<COUNT, STRIDE> {
    const COUNT: u16 = COUNT;
    const STRIDE: i32 = STRIDE;

    #[track_caller]
    #[inline(always)]
    fn assert_len_and_index(len: usize, index: Self::Index) {
        assert!(
            index < COUNT as usize,
            "Index out of range: {index} (array len: {COUNT})"
        );
        assert!(
            len + index <= COUNT as usize,
            "Array too long. At index {index}, the max len is {}",
            COUNT as usize - index,
        );
    }
}

#[doc(hidden)]
pub struct EnumRepeat<T, const STRIDE: i32>(PhantomData<T>);
impl<T: Clone + Into<i32>, const STRIDE: i32> Repeating for EnumRepeat<T, STRIDE> {
    type Index = T;

    #[inline(always)]
    fn calc_address<AddressType: Address>(start: AddressType, index: Self::Index) -> AddressType {
        let offset = index.into() * STRIDE;
        start.add(offset)
    }
}

#[doc(hidden)]
pub trait Address: Copy {
    fn add(self, val: i32) -> Self;
}

impl Address for u8 {
    fn add(self, val: i32) -> Self {
        (self as i32 + val).try_into().unwrap()
    }
}
impl Address for u16 {
    fn add(self, val: i32) -> Self {
        (self as i32 + val).try_into().unwrap()
    }
}
impl Address for u32 {
    fn add(self, val: i32) -> Self {
        self.checked_add_signed(val).unwrap()
    }
}
impl Address for u64 {
    fn add(self, val: i32) -> Self {
        self.checked_add_signed(val as i64).unwrap()
    }
}
impl Address for i8 {
    fn add(self, val: i32) -> Self {
        (self as i32 + val).try_into().unwrap()
    }
}
impl Address for i16 {
    fn add(self, val: i32) -> Self {
        (self as i32 + val).try_into().unwrap()
    }
}
impl Address for i32 {
    fn add(self, val: i32) -> Self {
        self + val
    }
}
impl Address for i64 {
    fn add(self, val: i32) -> Self {
        self + val as i64
    }
}
