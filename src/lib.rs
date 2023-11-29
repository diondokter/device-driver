#![cfg_attr(not(test), no_std)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use core::{
    convert::{TryFrom, TryInto},
    marker::PhantomData,
};

use bitvec::{field::BitField, slice::BitSlice};
use funty::Integral;

pub trait Register {
    const ZERO: Self;

    type AddressType;
    const ADDRESS: Self::AddressType;

    const SIZE_BITS: usize;
    type Endianness: EndiannessType;

    fn bits(&mut self) -> &mut BitSlice<u8>;

    fn default() -> Self
    where
        Self: Sized,
    {
        Self::ZERO
    }
}

const fn size_bytes<R: Register>() -> usize {
    if R::SIZE_BITS % 8 == 0 {
        R::SIZE_BITS / 8
    } else {
        R::SIZE_BITS / 8 + 1
    }
}

pub struct Field<'a, R: Register, DATA, BACKING, const START: usize, const END: usize> {
    register: &'a mut R,
    _phantom: PhantomData<(DATA, BACKING)>,
}

impl<'a, R: Register, DATA, BACKING, const START: usize, const END: usize>
    Field<'a, R, DATA, BACKING, START, END>
where
    DATA: TryFrom<BACKING> + Into<BACKING>,
    BACKING: Integral,
{
    fn new(register: &'a mut R) -> Self {
        Self {
            register,
            _phantom: PhantomData,
        }
    }

    pub fn write(self, data: DATA) -> &'a mut R {
        self.register.bits()[START..END].store_be(data.into());
        self.register
    }

    pub fn read(self) -> Result<DATA, <BACKING as TryInto<DATA>>::Error> {
        Ok(self.register.bits()[START..END]
            .load_be::<BACKING>()
            .try_into()?)
    }
}

pub trait EndiannessType {
    const LITTLE_ENDIAN: bool;
}

pub struct LittleEndian;
impl EndiannessType for LittleEndian {
    const LITTLE_ENDIAN: bool = true;
}
pub struct BigEndian;
impl EndiannessType for BigEndian {
    const LITTLE_ENDIAN: bool = false;
}

pub struct NativeEndian;
impl EndiannessType for NativeEndian {
    const LITTLE_ENDIAN: bool = cfg!(target_endian = "little");
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DeviceId {
        bits: BitArray<[u8; size_bytes::<Self>()]>,
    }

    impl DeviceId {
        pub fn manufacturer(&mut self) -> Field<'_, Self, u8, u8, 0, 8> {
            Field::new(self)
        }

        pub fn series(&mut self) -> Field<'_, Self, u8, u8, 8, 12> {
            Field::new(self)
        }
    }

    impl Register for DeviceId {
        const ZERO: Self = Self {
            bits: BitArray::ZERO,
        };

        type AddressType = u8;
        const ADDRESS: Self::AddressType = 55;

        const SIZE_BITS: usize = 12;
        type Endianness = LittleEndian;

        fn bits(&mut self) -> &mut BitSlice<u8> {
            &mut self.bits
        }
    }

    #[test]
    fn test_name() {
        let mut id = DeviceId::ZERO;

        id.manufacturer().write(12).series().write(5);

        assert_eq!(id.manufacturer().read().unwrap(), 12);
        assert_eq!(id.series().read().unwrap(), 5);
    }
}
