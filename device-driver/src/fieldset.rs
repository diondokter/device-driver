#![allow(unused)]

use crate::FieldsetMetadata;

/// # Safety
/// Implers of this trait will get their memory changed through a byte slice.
/// So make sure the type has a stable ABI and has no padding bits.
pub unsafe trait Fieldset: Sized {
    /// Metadata describing some properties of the fieldset
    const METADATA: FieldsetMetadata;
    /// Get a zero-initialized instance of the fieldset
    const ZERO: Self;

    /// Get the fieldset as a slice
    fn as_slice(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                (self as *const Self).cast::<u8>(),
                core::mem::size_of::<Self>(),
            )
        }
    }
    /// Get the fieldset as a mutable slice
    fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                (self as *mut Self).cast::<u8>(),
                core::mem::size_of::<Self>(),
            )
        }
    }
}

unsafe impl<T: Fieldset, const N: usize> Fieldset for [T; N] {
    const METADATA: FieldsetMetadata = T::METADATA;
    const ZERO: Self = [T::ZERO; N];
}

#[doc(hidden)]
pub trait ToTuple {
    type Tuple;
    fn to_tuple(self) -> Self::Tuple;
}

#[doc(hidden)]
pub trait Append<T> {
    type Appended;
    fn append(self, val: T) -> Self::Appended;
}

impl<A> Append<A> for () {
    type Appended = Fs1<A>;

    fn append(self, val: A) -> Self::Appended {
        Fs1(val)
    }
}

macro_rules! impl_append {
    ($name:ident -> None, $(($tname:ident: $tnum:tt)),+) => {
        // No append impl
    };
    ($name:ident -> $next:ident, $(($tname:ident: $tnum:tt)),+) => {
        impl<$($tname),*, NEW> Append<NEW> for $name<$($tname),*> {
            type Appended = $next<$($tname),* , NEW>;
            fn append(self, val: NEW) -> Self::Appended {
                $next($(self.$tnum),*, val)
            }
        }
    };
}

macro_rules! create_fsn {
    ($name:ident -> $next:ident, $(($tname:ident: $tnum:tt)),+) => {
        /// Combined fieldsets
        #[derive(Debug)]
        #[repr(C)]
        #[doc(hidden)]
        pub struct $name<$($tname),*>($($tname),*);

        unsafe impl<$($tname: Fieldset),*> Fieldset for $name<$($tname),*> {
            const METADATA: FieldsetMetadata = A::METADATA;
            const ZERO: Self = Self($($tname::ZERO),*);
        }

        impl<$($tname),*> ToTuple for $name<$($tname),*> {
            type Tuple = ($($tname),*);
            fn to_tuple(self) -> Self::Tuple {
                (
                    $(self.$tnum),*
                )
            }
        }
        impl<'a, $($tname),*> ToTuple for &'a mut $name<$($tname),*> {
            type Tuple = ($(&'a mut $tname),*);
            fn to_tuple(self) -> Self::Tuple {
                (
                    $(&mut self.$tnum),*
                )
            }
        }

        impl_append!($name -> $next, $(($tname: $tnum)),+);
    }
}

create_fsn!(Fs1  -> Fs2, (A: 0));
create_fsn!(Fs2  -> Fs3, (A: 0), (B: 1));
create_fsn!(Fs3  -> Fs4, (A: 0), (B: 1), (C: 2));
create_fsn!(Fs4  -> Fs5, (A: 0), (B: 1), (C: 2), (D: 3));
create_fsn!(Fs5  -> Fs6, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4));
create_fsn!(Fs6  -> Fs7, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5));
create_fsn!(Fs7  -> Fs8, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6));
create_fsn!(Fs8  -> Fs9, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7));
create_fsn!(Fs9  -> Fs10, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8));
create_fsn!(Fs10 -> Fs11, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9));
create_fsn!(Fs11 -> Fs12, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10));
create_fsn!(Fs12 -> Fs13, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11));
create_fsn!(Fs13 -> Fs14, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12));
create_fsn!(Fs14 -> Fs15, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13));
create_fsn!(Fs15 -> Fs16, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14));
create_fsn!(Fs16 -> Fs17, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14), (P: 15));
create_fsn!(Fs17 -> Fs18, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14), (P: 15), (Q: 16));
create_fsn!(Fs18 -> Fs19, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14), (P: 15), (Q: 16), (R: 17));
create_fsn!(Fs19 -> Fs20, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14), (P: 15), (Q: 16), (R: 17), (S: 18));
create_fsn!(Fs20 -> Fs21, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14), (P: 15), (Q: 16), (R: 17), (S: 18), (T: 19));
create_fsn!(Fs21 -> Fs22, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14), (P: 15), (Q: 16), (R: 17), (S: 18), (T: 19), (U: 20));
create_fsn!(Fs22 -> Fs23, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14), (P: 15), (Q: 16), (R: 17), (S: 18), (T: 19), (U: 20), (V: 21));
create_fsn!(Fs23 -> Fs24, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14), (P: 15), (Q: 16), (R: 17), (S: 18), (T: 19), (U: 20), (V: 21), (W: 22));
create_fsn!(Fs24 -> Fs25, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14), (P: 15), (Q: 16), (R: 17), (S: 18), (T: 19), (U: 20), (V: 21), (W: 22), (X: 23));
create_fsn!(Fs25 -> Fs26, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14), (P: 15), (Q: 16), (R: 17), (S: 18), (T: 19), (U: 20), (V: 21), (W: 22), (X: 23), (Y: 24));
create_fsn!(Fs26 -> None, (A: 0), (B: 1), (C: 2), (D: 3), (E: 4), (F: 5), (G: 6), (H: 7), (I: 8), (J: 9), (K: 10), (L: 11), (M: 12), (N: 13), (O: 14), (P: 15), (Q: 16), (R: 17), (S: 18), (T: 19), (U: 20), (V: 21), (W: 22), (X: 23), (Y: 24), (Z: 25));
