#![allow(async_fn_in_trait)]
#![cfg_attr(not(test), no_std)]

use core::{
    convert::{TryFrom, TryInto},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub use bitvec;
pub use device_driver_macros::*;
pub use funty;
pub use num_enum;

use bitvec::{array::BitArray, field::BitField};
use funty::Integral;

pub trait RegisterDevice {
    type Error;
    type AddressType;

    fn write_register<R, const SIZE_BYTES: usize>(
        &mut self,
        data: &BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error>
    where
        R: Register<SIZE_BYTES, AddressType = Self::AddressType>;

    fn read_register<R, const SIZE_BYTES: usize>(
        &mut self,
        data: &mut BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error>
    where
        R: Register<SIZE_BYTES, AddressType = Self::AddressType>;
}

pub trait AsyncRegisterDevice {
    type Error;
    type AddressType;

    async fn write_register<R, const SIZE_BYTES: usize>(
        &mut self,
        data: &BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error>
    where
        R: Register<SIZE_BYTES, AddressType = Self::AddressType>;

    async fn read_register<R, const SIZE_BYTES: usize>(
        &mut self,
        data: &mut BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error>
    where
        R: Register<SIZE_BYTES, AddressType = Self::AddressType>;
}

pub trait Register<const SIZE_BYTES: usize> {
    const ZERO: Self;

    type AddressType;
    const ADDRESS: Self::AddressType;

    type RWCapability;
    const SIZE_BITS: usize;

    type WriteFields: From<Self> + Into<Self> + Deref<Target = Self> + DerefMut
    where
        Self: Sized;
    type ReadFields: From<Self> + Into<Self> + Deref<Target = Self> + DerefMut
    where
        Self: Sized;

    fn bits_mut(&mut self) -> &mut BitArray<[u8; SIZE_BYTES]>;
    fn bits(&self) -> &BitArray<[u8; SIZE_BYTES]>;

    fn default() -> Self
    where
        Self: Sized,
    {
        Self::ZERO
    }
}

pub struct RegisterOperation<'a, D, R, const SIZE_BYTES: usize>
where
    R: Register<SIZE_BYTES>,
{
    device: &'a mut D,
    _phantom: PhantomData<R>,
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    R: Register<SIZE_BYTES>,
{
    pub fn new(device: &'a mut D) -> Self {
        Self {
            device,
            _phantom: PhantomData,
        }
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: RegisterDevice<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWCapability: WriteCapability,
{
    pub fn write(
        &mut self,
        f: impl FnOnce(&mut R::WriteFields) -> &mut R::WriteFields,
    ) -> Result<(), D::Error> {
        let mut register = R::ZERO.into();
        f(&mut register);
        self.device
            .write_register::<R, SIZE_BYTES>(register.into().bits())
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: RegisterDevice<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWCapability: ReadCapability,
{
    pub fn read(&mut self) -> Result<R::ReadFields, D::Error> {
        let mut register = R::ZERO;
        self.device
            .read_register::<R, SIZE_BYTES>(register.bits_mut())?;
        Ok(register.into())
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: RegisterDevice<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWCapability: ReadCapability + WriteCapability,
{
    pub fn modify(
        &mut self,
        f: impl FnOnce(&mut R::WriteFields) -> &mut R::WriteFields,
    ) -> Result<(), D::Error> {
        let mut register = self.read()?.into().into();
        f(&mut register);
        self.device
            .write_register::<R, SIZE_BYTES>(register.into().bits())
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: AsyncRegisterDevice<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWCapability: WriteCapability,
{
    pub async fn write_async(&mut self, f: impl FnOnce(&mut R) -> &mut R) -> Result<(), D::Error> {
        let mut register = R::ZERO;
        f(&mut register);
        self.device
            .write_register::<R, SIZE_BYTES>(register.bits())
            .await
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: AsyncRegisterDevice<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWCapability: ReadCapability,
{
    pub async fn read_async(&mut self) -> Result<R, D::Error> {
        let mut register = R::ZERO;
        self.device
            .read_register::<R, SIZE_BYTES>(register.bits_mut())
            .await?;
        Ok(register)
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: AsyncRegisterDevice<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWCapability: ReadCapability + WriteCapability,
{
    pub async fn modify_async(&mut self, f: impl FnOnce(&mut R) -> &mut R) -> Result<(), D::Error> {
        let mut register = self.read_async().await?;
        f(&mut register);
        self.device
            .write_register::<R, SIZE_BYTES>(register.bits())
            .await
    }
}

pub fn read_field<
    RD,
    R,
    DATA,
    BACKING,
    const START: usize,
    const END: usize,
    const SIZE_BYTES: usize,
>(
    register: &R,
) -> Result<DATA, <BACKING as TryInto<DATA>>::Error>
where
    RD: Deref<Target = R>,
    R: Register<SIZE_BYTES>,
    DATA: TryFrom<BACKING> + Into<BACKING>,
    BACKING: Integral,
{
    register.bits()[START..END].load_be::<BACKING>().try_into()
}

pub fn write_field<
    RD,
    R,
    DATA,
    BACKING,
    const START: usize,
    const END: usize,
    const SIZE_BYTES: usize,
>(
    register: &mut RD,
    data: DATA,
) -> &mut RD
where
    RD: DerefMut<Target = R>,
    R: Register<SIZE_BYTES>,
    DATA: TryFrom<BACKING> + Into<BACKING>,
    BACKING: Integral,
{
    register.bits_mut()[START..END].store_be(data.into());
    register
}

pub struct WriteOnly;
pub struct ReadOnly;
pub struct ReadWrite;

pub trait ReadCapability {}
pub trait WriteCapability {}

impl WriteCapability for WriteOnly {}
impl ReadCapability for ReadOnly {}
impl WriteCapability for ReadWrite {}
impl ReadCapability for ReadWrite {}
