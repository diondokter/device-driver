#![cfg_attr(not(test), no_std)]

pub trait Register {
    type AddressType;
    const ADDRESS: Self::AddressType;

    const SIZE_BITS: usize;
    type Endianness: EndiannessType;
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
