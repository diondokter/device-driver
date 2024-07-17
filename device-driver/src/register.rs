use bitvec::{array::BitArray, field::BitField};
use core::{
    convert::{TryFrom, TryInto},
    marker::PhantomData,
    ops::{Add, Deref, DerefMut},
};
use funty::Integral;

use crate::{ClearCapability, ReadCapability, WriteCapability};

/// A trait for devices that have addressable registers.
pub trait AddressableDevice {
    /// The address type used by this interface. Should likely be an integer.
    type AddressType;
}

/// A trait to represent the interface to the device.
///
/// This is called to write to and read from registers.
pub trait RegisterDevice: AddressableDevice {
    /// The error type
    type Error;

    /// Write the given data to the register.
    ///
    /// The address and the length of the register is descriped with the constant in the `R` [Register] type.
    /// The data can made into a normal slice by calling `as_raw_slice` on it.
    fn write_register<const SIZE_BYTES: usize>(
        &mut self,
        address: Self::AddressType,
        data: &BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error>;

    /// Read the data from the register into the given buffer.
    ///
    /// The address and the length of the register is descriped with the constant in the `R` [Register] type.
    /// The data can made into a normal slice by calling `as_raw_slice` on it.
    fn read_register<const SIZE_BYTES: usize>(
        &mut self,
        address: Self::AddressType,
        data: &mut BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error>;
}

/// A trait to represent the interface to the device.
///
/// This is called to asynchronously write to and read from registers.
pub trait AsyncRegisterDevice: AddressableDevice {
    /// The error type
    type Error;

    /// Write the given data to the register asynchronously.
    ///
    /// The address and the length of the register is descriped with the constant in the `R` [Register] type.
    /// The data can made into a normal slice by calling `as_raw_slice` on it.
    async fn write_register<const SIZE_BYTES: usize>(
        &mut self,
        address: Self::AddressType,
        data: &BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error>;

    /// Read the data from the register asynchronously into the given buffer.
    ///
    /// The address and the length of the register is descriped with the constant in the `R` [Register] type.
    /// The data can made into a normal slice by calling `as_raw_slice` on it.
    async fn read_register<const SIZE_BYTES: usize>(
        &mut self,
        address: Self::AddressType,
        data: &mut BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error>;
}

/// A trait to represent a block of registers.
pub trait RegisterBlock {
    /// The type of device this register block belongs to.
    type Device: AddressableDevice;

    /// The base address to use for registers in this register block.
    ///
    /// This base address will be added to the register's address when reading or writing the register to the parent device.
    const BASE_ADDR: <Self::Device as AddressableDevice>::AddressType;

    /// The device this register block belongs to.
    fn dev(&mut self) -> &mut Self::Device;
}

impl<T: RegisterBlock> AddressableDevice for T {
    type AddressType = <T::Device as AddressableDevice>::AddressType;
}

impl<T: RegisterBlock> RegisterDevice for T
where
    T::Device: RegisterDevice,
    <T::Device as AddressableDevice>::AddressType: Add<
        <T::Device as AddressableDevice>::AddressType,
        Output = <T::Device as AddressableDevice>::AddressType,
    >,
{
    type Error = <T::Device as RegisterDevice>::Error;

    fn write_register<const SIZE_BYTES: usize>(
        &mut self,
        address: Self::AddressType,
        data: &BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error> {
        self.dev()
            .write_register::<SIZE_BYTES>(address + Self::BASE_ADDR, data)
    }

    fn read_register<const SIZE_BYTES: usize>(
        &mut self,
        address: Self::AddressType,
        data: &mut BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error> {
        self.dev()
            .read_register::<SIZE_BYTES>(address + Self::BASE_ADDR, data)
    }
}

impl<T: RegisterBlock> AsyncRegisterDevice for T
where
    T::Device: AsyncRegisterDevice,
    <T::Device as AddressableDevice>::AddressType: Add<
        <T::Device as AddressableDevice>::AddressType,
        Output = <T::Device as AddressableDevice>::AddressType,
    >,
{
    type Error = <T::Device as AsyncRegisterDevice>::Error;

    async fn write_register<const SIZE_BYTES: usize>(
        &mut self,
        address: Self::AddressType,
        data: &BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error> {
        self.dev()
            .write_register::<SIZE_BYTES>(address + Self::BASE_ADDR, data)
            .await
    }

    async fn read_register<const SIZE_BYTES: usize>(
        &mut self,
        address: Self::AddressType,
        data: &mut BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error> {
        self.dev()
            .read_register::<SIZE_BYTES>(address + Self::BASE_ADDR, data)
            .await
    }
}

/// The abstraction and description of a register.
///
/// This is meant to be implemented by the macros.
pub trait Register<const SIZE_BYTES: usize> {
    /// The all-zero bits representation of this register
    const ZERO: Self;

    /// The address type of this register abstraction
    type AddressType;
    /// The address value of where this register is stored on the device
    const ADDRESS: Self::AddressType;

    /// Typestate value of how this register can be used.
    ///
    /// Should be one of:
    /// - [WriteOnly]
    /// - [ReadOnly]
    /// - [ClearOnly]
    /// - [ReadWrite]
    /// - [ReadClear]
    type RWType;

    /// The size of the register in bits
    const SIZE_BITS: usize;

    /// The type of the writer `W` type associated with this register
    type WriteFields: From<Self> + Into<Self> + Deref<Target = Self> + DerefMut
    where
        Self: Sized;
    /// The type of the reader `R` type associated with this register
    type ReadFields: From<Self> + Into<Self> + Deref<Target = Self> + DerefMut
    where
        Self: Sized;

    /// Is the data of this register little endian?
    /// False by default when not specified by the end-user
    const LE: bool;

    /// Get mutable access to the local representation of the register bits
    fn bits_mut(&mut self) -> &mut BitArray<[u8; SIZE_BYTES]>;
    /// Get access to the local representation of the register bits
    fn bits(&self) -> &BitArray<[u8; SIZE_BYTES]>;

    /// The 'default'/reset value of the register.
    ///
    /// Optional. Standardly implemented as returning [Self::ZERO].
    fn reset_value() -> Self
    where
        Self: Sized,
    {
        Self::ZERO
    }
}

/// Object that performs actions on the device in the context of a register
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
    #[doc(hidden)]
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
    R::RWType: WriteCapability,
{
    /// Write to the register.
    ///
    /// The closure is given the write object initialized to the reset value of the register.
    /// If no reset value is specified for this register, this function is the same as [Self::write_with_zero].
    pub fn write(
        &mut self,
        f: impl FnOnce(&mut R::WriteFields) -> &mut R::WriteFields,
    ) -> Result<(), D::Error> {
        let mut register = R::reset_value().into();
        f(&mut register);
        self.device
            .write_register::<SIZE_BYTES>(R::ADDRESS, register.bits())
    }

    /// Write to the register.
    ///
    /// The closure is given the write object initialized to all zero.
    pub fn write_with_zero(
        &mut self,
        f: impl FnOnce(&mut R::WriteFields) -> &mut R::WriteFields,
    ) -> Result<(), D::Error> {
        let mut register = R::ZERO.into();
        f(&mut register);
        self.device
            .write_register::<SIZE_BYTES>(R::ADDRESS, register.bits())
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: RegisterDevice<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWType: ReadCapability,
{
    /// Read the register from the device
    pub fn read(&mut self) -> Result<R::ReadFields, D::Error> {
        let mut register = R::ZERO;
        self.device
            .read_register::<SIZE_BYTES>(R::ADDRESS, register.bits_mut())?;
        Ok(register.into())
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: RegisterDevice<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWType: ReadCapability + WriteCapability,
{
    /// Modify the existing register value.
    ///
    /// The register is read, the value is then passed to the closure for making changes.
    /// The result is then written back to the device.
    pub fn modify(
        &mut self,
        f: impl FnOnce(&mut R::WriteFields) -> &mut R::WriteFields,
    ) -> Result<(), D::Error> {
        let mut register = self.read()?.into().into();
        f(&mut register);
        self.device
            .write_register::<SIZE_BYTES>(R::ADDRESS, register.into().bits())
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: RegisterDevice<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWType: ClearCapability,
{
    /// Write the reset value (or zero if the reset value is specified) to the register
    pub fn clear(&mut self) -> Result<(), D::Error> {
        self.device
            .write_register::<SIZE_BYTES>(R::ADDRESS, R::reset_value().bits())
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: AsyncRegisterDevice<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWType: WriteCapability,
{
    /// Write to the register.
    ///
    /// The closure is given the write object initialized to the reset value of the register.
    /// If no reset value is specified for this register, this function is the same as [Self::write_with_zero].
    pub async fn write_async(
        &mut self,
        f: impl FnOnce(&mut R::WriteFields) -> &mut R::WriteFields,
    ) -> Result<(), D::Error> {
        let mut register = R::reset_value().into();
        f(&mut register);
        self.device
            .write_register::<SIZE_BYTES>(R::ADDRESS, register.bits())
            .await
    }

    /// Write to the register.
    ///
    /// The closure is given the write object initialized to all zero.
    pub async fn write_with_zero_async(
        &mut self,
        f: impl FnOnce(&mut R::WriteFields) -> &mut R::WriteFields,
    ) -> Result<(), D::Error> {
        let mut register = R::ZERO.into();
        f(&mut register);
        self.device
            .write_register::<SIZE_BYTES>(R::ADDRESS, register.bits())
            .await
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: AsyncRegisterDevice<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWType: ReadCapability,
{
    /// Read the register from the device
    pub async fn read_async(&mut self) -> Result<R::ReadFields, D::Error> {
        let mut register = R::ZERO;
        self.device
            .read_register::<SIZE_BYTES>(R::ADDRESS, register.bits_mut())
            .await?;
        Ok(register.into())
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: AsyncRegisterDevice<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWType: ReadCapability + WriteCapability,
{
    /// Modify the existing register value.
    ///
    /// The register is read, the value is then passed to the closure for making changes.
    /// The result is then written back to the device.
    pub async fn modify_async(
        &mut self,
        f: impl FnOnce(&mut R::WriteFields) -> &mut R::WriteFields,
    ) -> Result<(), D::Error> {
        let mut register = self.read_async().await?.into().into();
        f(&mut register);
        self.device
            .write_register::<SIZE_BYTES>(R::ADDRESS, register.into().bits())
            .await
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: AsyncRegisterDevice<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWType: ClearCapability,
{
    /// Write the reset value (or zero if the reset value is specified) to the register
    pub async fn clear_async(&mut self) -> Result<(), D::Error> {
        self.device
            .write_register::<SIZE_BYTES>(R::ADDRESS, R::reset_value().bits())
            .await
    }
}

#[doc(hidden)]
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
    if R::LE {
        register.bits()[START..END].load_le::<BACKING>().try_into()
    } else {
        register.bits()[START..END].load_be::<BACKING>().try_into()
    }
}

#[doc(hidden)]
pub fn read_field_strict<
    RD,
    R,
    DATA,
    BACKING,
    const START: usize,
    const END: usize,
    const SIZE_BYTES: usize,
>(
    register: &R,
) -> DATA
where
    RD: Deref<Target = R>,
    R: Register<SIZE_BYTES>,
    DATA: From<BACKING> + Into<BACKING>,
    BACKING: Integral,
{
    if R::LE {
        register.bits()[START..END].load_le::<BACKING>().into()
    } else {
        register.bits()[START..END].load_be::<BACKING>().into()
    }
}

#[doc(hidden)]
pub fn read_field_no_convert<
    RD,
    R,
    BACKING,
    const START: usize,
    const END: usize,
    const SIZE_BYTES: usize,
>(
    register: &R,
) -> BACKING
where
    RD: Deref<Target = R>,
    R: Register<SIZE_BYTES>,
    BACKING: Integral,
{
    if R::LE {
        register.bits()[START..END].load_le::<BACKING>()
    } else {
        register.bits()[START..END].load_be::<BACKING>()
    }
}

#[doc(hidden)]
pub fn read_field_bool<RD, R, DATA, const START: usize, const SIZE_BYTES: usize>(
    register: &R,
) -> Result<DATA, <bool as TryInto<DATA>>::Error>
where
    RD: Deref<Target = R>,
    R: Register<SIZE_BYTES>,
    DATA: TryFrom<bool> + Into<bool>,
{
    register.bits()[START].try_into()
}

#[doc(hidden)]
pub fn read_field_bool_strict<RD, R, DATA, const START: usize, const SIZE_BYTES: usize>(
    register: &R,
) -> DATA
where
    RD: Deref<Target = R>,
    R: Register<SIZE_BYTES>,
    DATA: From<bool> + Into<bool>,
{
    register.bits()[START].into()
}

#[doc(hidden)]
pub fn read_field_bool_no_convert<RD, R, const START: usize, const SIZE_BYTES: usize>(
    register: &R,
) -> bool
where
    RD: Deref<Target = R>,
    R: Register<SIZE_BYTES>,
{
    register.bits()[START]
}

#[doc(hidden)]
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
    if R::LE {
        register.bits_mut()[START..END].store_le(data.into());
    } else {
        register.bits_mut()[START..END].store_be(data.into());
    }

    register
}

#[doc(hidden)]
pub fn write_field_no_convert<
    RD,
    R,
    BACKING,
    const START: usize,
    const END: usize,
    const SIZE_BYTES: usize,
>(
    register: &mut RD,
    data: BACKING,
) -> &mut RD
where
    RD: DerefMut<Target = R>,
    R: Register<SIZE_BYTES>,
    BACKING: Integral,
{
    if R::LE {
        register.bits_mut()[START..END].store_le(data);
    } else {
        register.bits_mut()[START..END].store_be(data);
    }

    register
}

#[doc(hidden)]
pub fn write_field_bool<RD, R, DATA, const START: usize, const SIZE_BYTES: usize>(
    register: &mut RD,
    data: DATA,
) -> &mut RD
where
    RD: DerefMut<Target = R>,
    R: Register<SIZE_BYTES>,
    DATA: TryFrom<bool> + Into<bool>,
{
    register.bits_mut().set(START, data.into());
    register
}

#[doc(hidden)]
pub fn write_field_bool_no_convert<RD, R, const START: usize, const SIZE_BYTES: usize>(
    register: &mut RD,
    data: bool,
) -> &mut RD
where
    RD: DerefMut<Target = R>,
    R: Register<SIZE_BYTES>,
{
    register.bits_mut().set(START, data);
    register
}
