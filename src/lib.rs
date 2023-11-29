#![cfg_attr(not(test), no_std)]

use core::{
    convert::{TryFrom, TryInto},
    marker::PhantomData,
};

use bitvec::{array::BitArray, field::BitField};
use funty::Integral;

pub trait Device {
    type Error;
    type AddressType;

    fn write<R, const SIZE_BYTES: usize>(
        &mut self,
        data: &BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error>
    where
        R: Register<SIZE_BYTES, AddressType = Self::AddressType>;
    fn read<R, const SIZE_BYTES: usize>(
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

    fn bits(&mut self) -> &mut BitArray<[u8; SIZE_BYTES]>;

    fn default() -> Self
    where
        Self: Sized,
    {
        Self::ZERO
    }
}

pub struct RegisterOperation<'a, D: Device, R: Register<SIZE_BYTES>, const SIZE_BYTES: usize> {
    device: &'a mut D,
    _phantom: PhantomData<R>,
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: Device<AddressType = R::AddressType>,
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
    D: Device<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWCapability: WriteCapability,
{
    pub fn write(&mut self, f: impl FnOnce(&mut R) -> &mut R) -> Result<(), D::Error> {
        let mut register = R::ZERO;
        f(&mut register);
        self.device.write::<R, SIZE_BYTES>(register.bits())
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: Device<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWCapability: ReadCapability,
{
    pub fn read(&mut self) -> Result<R, D::Error> {
        let mut register = R::ZERO;
        self.device.read::<R, SIZE_BYTES>(register.bits())?;
        Ok(register)
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: Device<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWCapability: ReadCapability + WriteCapability,
{
    pub fn modify(&mut self, f: impl FnOnce(&mut R) -> &mut R) -> Result<(), D::Error> {
        let mut register = self.read()?;
        f(&mut register);
        self.device.write::<R, SIZE_BYTES>(register.bits())
    }
}

pub struct Field<
    'a,
    R: Register<SIZE_BYTES>,
    DATA,
    BACKING,
    const START: usize,
    const END: usize,
    const SIZE_BYTES: usize,
> {
    register: &'a mut R,
    _phantom: PhantomData<(DATA, BACKING)>,
}

impl<
        'a,
        R: Register<SIZE_BYTES>,
        DATA,
        BACKING,
        const START: usize,
        const END: usize,
        const SIZE_BYTES: usize,
    > Field<'a, R, DATA, BACKING, START, END, SIZE_BYTES>
where
    DATA: TryFrom<BACKING> + Into<BACKING>,
    BACKING: Integral,
{
    pub fn new(register: &'a mut R) -> Self {
        Self {
            register,
            _phantom: PhantomData,
        }
    }

    pub fn set(self, data: DATA) -> &'a mut R {
        self.register.bits()[START..END].store_be(data.into());
        self.register
    }

    pub fn get(self) -> Result<DATA, <BACKING as TryInto<DATA>>::Error> {
        self.register.bits()[START..END]
            .load_be::<BACKING>()
            .try_into()
    }
}

pub struct Write;
pub struct Read;
pub struct ReadWrite;

pub trait ReadCapability {}
pub trait WriteCapability {}

impl WriteCapability for Write {}
impl ReadCapability for Read {}
impl WriteCapability for ReadWrite {}
impl ReadCapability for ReadWrite {}

#[cfg(test)]
pub mod tests {
    use super::*;

    struct TestDevice {
        device_memory: [u8; 128],
    }

    impl Device for TestDevice {
        type Error = ();
        type AddressType = usize;

        fn write<R, const SIZE_BYTES: usize>(
            &mut self,
            data: &BitArray<[u8; SIZE_BYTES]>,
        ) -> Result<(), Self::Error>
        where
            R: Register<SIZE_BYTES, AddressType = Self::AddressType>,
        {
            self.device_memory[R::ADDRESS..][..SIZE_BYTES].copy_from_slice(data.as_raw_slice());

            Ok(())
        }

        fn read<R, const SIZE_BYTES: usize>(
            &mut self,
            data: &mut BitArray<[u8; SIZE_BYTES]>,
        ) -> Result<(), Self::Error>
        where
            R: Register<SIZE_BYTES, AddressType = Self::AddressType>,
        {
            data.as_raw_mut_slice()
                .copy_from_slice(&self.device_memory[R::ADDRESS..][..SIZE_BYTES]);
            Ok(())
        }
    }

    impl TestDevice {
        pub fn new() -> Self {
            // Normally we'd take like a SPI here or something
            Self {
                device_memory: [0; 128],
            }
        }

        pub fn device_id(
            &mut self,
        ) -> RegisterOperation<'_, Self, DeviceId, { DeviceId::SIZE_BYTES }> {
            RegisterOperation::new(self)
        }
    }

    struct DeviceId {
        bits: BitArray<[u8; Self::SIZE_BYTES]>,
    }

    impl DeviceId {
        pub const SIZE_BYTES: usize = 2;

        pub fn manufacturer(&mut self) -> Field<'_, Self, u8, u8, 0, 8, { Self::SIZE_BYTES }> {
            Field::new(self)
        }

        pub fn series(&mut self) -> Field<'_, Self, Series, u8, 8, 12, { Self::SIZE_BYTES }> {
            Field::new(self)
        }
    }

    #[derive(num_enum::TryFromPrimitive, num_enum::IntoPrimitive, PartialEq, Debug)]
    #[repr(u8)]
    enum Series {
        A,
        B,
        C,
    }

    impl Register<{ Self::SIZE_BYTES }> for DeviceId {
        const ZERO: Self = Self {
            bits: BitArray::ZERO,
        };

        type AddressType = usize;
        const ADDRESS: Self::AddressType = 0;

        type RWCapability = ReadWrite;

        fn bits(&mut self) -> &mut BitArray<[u8; Self::SIZE_BYTES]> {
            &mut self.bits
        }
    }

    #[test]
    pub fn test_name() {
        let mut test_device = TestDevice::new();

        test_device
            .device_id()
            .write(|w| w.manufacturer().set(12))
            .unwrap();
        test_device
            .device_id()
            .modify(|w| w.series().set(Series::B))
            .unwrap();

        let mut id = test_device.device_id().read().unwrap();
        assert_eq!(id.manufacturer().get().unwrap(), 12);
        assert_eq!(id.series().get().unwrap(), Series::B);
    }
}
