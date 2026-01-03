#![allow(async_fn_in_trait)]
#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]
#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

use core::fmt::{Debug, Display};

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
pub unsafe trait FieldSet: Default {
    /// The size of the field set in number of bits
    const SIZE_BITS: u32;

    fn get_inner_buffer(&self) -> &[u8];
    fn get_inner_buffer_mut(&mut self) -> &mut [u8];
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

/// # Safety
///
/// May only be implemented on type that you can safely implement [FsSet::as_slice_mut] for
pub unsafe trait FsSet: Sized {
    type Value;
    type ValueMut<'a>
    where
        Self: 'a;
    type Next<T: FieldSet>;

    fn push<T: FieldSet>(self, val: T) -> Self::Next<T>;
    fn to_value(self) -> Self::Value;
    fn as_value_mut(&mut self) -> Self::ValueMut<'_>;

    fn as_slice_mut(&mut self) -> &mut [u8] {
        // Safety: Trait is only implemented on types that can do this.
        unsafe {
            let len = core::mem::size_of::<Self>();
            let ptr = self as *mut Self;
            core::slice::from_raw_parts_mut(ptr.cast(), len)
        }
    }
}

unsafe impl FsSet for () {
    type Value = ();
    type ValueMut<'a> = &'a mut ();
    type Next<T: FieldSet> = T;

    fn push<T: FieldSet>(self, val: T) -> Self::Next<T> {
        val
    }

    fn to_value(self) -> Self::Value {
        ()
    }

    fn as_value_mut(&mut self) -> Self::ValueMut<'_> {
        self
    }
}

unsafe impl<A: FieldSet> FsSet for A {
    type Value = A;
    type ValueMut<'a>
        = &'a mut A
    where
        A: 'a;
    type Next<T: FieldSet> = Fs2<A, T>;

    fn push<T: FieldSet>(self, val: T) -> Self::Next<T> {
        Fs2(self, val)
    }

    fn to_value(self) -> Self::Value {
        self
    }

    fn as_value_mut(&mut self) -> Self::ValueMut<'_> {
        self
    }
}

macro_rules! create_fs {
    ($name:ident -> $name_next:ident, $(($tname:ident: $tnum:tt)),+) => {
        #[derive(Debug)]
        #[repr(C)]
        pub struct $name<$($tname: FieldSet),*>($($tname),*);

        unsafe impl<$($tname: FieldSet),*> FsSet for $name<$($tname),*> {
            type Value = ($($tname),*);
            type ValueMut<'a>
                = ($(&'a mut $tname),*)
            where
                $($tname: 'a),*;
            type Next<Next: FieldSet> = $name_next<$($tname),*, Next>;

            fn push<Next: FieldSet>(self, val: Next) -> Self::Next<Next> {
                $name_next($(self.$tnum),*, val)
            }

            fn to_value(self) -> Self::Value {
                ($(self.$tnum),*)
            }

            fn as_value_mut(&mut self) -> Self::ValueMut<'_> {
                ($(&mut self.$tnum),*)
            }
        }
    };
    ($name:ident -> !, $(($tname:ident: $tnum:tt)),+) => {
        #[derive(Debug)]
        #[repr(C)]
        pub struct $name<$($tname: FieldSet),*>($($tname),*);

        unsafe impl<$($tname: FieldSet),*> FsSet for $name<$($tname),*> {
            type Value = ($($tname),*);
            type ValueMut<'a>
                = ($(&'a mut $tname),*)
            where
                $($tname: 'a),*;
            type Next<Next: FieldSet> = core::convert::Infallible;

            fn push<Next: FieldSet>(self, _val: Next) -> Self::Next<Next> {
                panic!()
            }

            fn to_value(self) -> Self::Value {
                ($(self.$tnum),*)
            }

            fn as_value_mut(&mut self) -> Self::ValueMut<'_> {
                ($(&mut self.$tnum),*)
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
