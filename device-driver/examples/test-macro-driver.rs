use bitvec::array::BitArray;
use device_driver::{AsyncRegisterDevice, Register, RegisterDevice};

pub struct TestDevice<const SIZE: usize> {
    device_memory: [u8; 128],
}

impl<const SIZE: usize> RegisterDevice for TestDevice<SIZE> {
    type Error = ();
    type AddressType = u8;

    fn write_register<R, const SIZE_BYTES: usize>(
        &mut self,
        data: &BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error>
    where
        R: Register<SIZE_BYTES, AddressType = Self::AddressType>,
    {
        self.device_memory[R::ADDRESS as usize..][..SIZE_BYTES]
            .copy_from_slice(data.as_raw_slice());

        Ok(())
    }

    fn read_register<R, const SIZE_BYTES: usize>(
        &mut self,
        data: &mut BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error>
    where
        R: Register<SIZE_BYTES, AddressType = Self::AddressType>,
    {
        data.as_raw_mut_slice()
            .copy_from_slice(&self.device_memory[R::ADDRESS as usize..][..SIZE_BYTES]);
        Ok(())
    }
}

impl<const SIZE: usize> AsyncRegisterDevice for TestDevice<SIZE> {
    type Error = ();
    type AddressType = u8;

    async fn write_register<R, const SIZE_BYTES: usize>(
        &mut self,
        data: &BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error>
    where
        R: Register<SIZE_BYTES, AddressType = Self::AddressType>,
    {
        self.device_memory[R::ADDRESS as usize..][..SIZE_BYTES]
            .copy_from_slice(data.as_raw_slice());

        Ok(())
    }

    async fn read_register<R, const SIZE_BYTES: usize>(
        &mut self,
        data: &mut BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error>
    where
        R: Register<SIZE_BYTES, AddressType = Self::AddressType>,
    {
        data.as_raw_mut_slice()
            .copy_from_slice(&self.device_memory[R::ADDRESS as usize..][..SIZE_BYTES]);
        Ok(())
    }
}

impl<const SIZE: usize> TestDevice<SIZE> {
    pub fn new() -> Self {
        // Normally we'd take like a SPI here or something
        Self {
            device_memory: [0; 128],
        }
    }
}

pub mod registers {
    use super::*;

    device_driver_macros::implement_registers!(
        impl<const SIZE: usize> TestDevice<SIZE> {
            register Id {
                type RWCapability = ReadOnly;
                const ADDRESS: usize = 12;
                const SIZE_BYTES: usize = 3;

                manufacturer: u16 as Manufacturer = 0..16,
                version: u8 = 16..20,
                edition: u8 as enum Edition {
                    One = 1,
                    Two,
                    Five = 5,
                } = 20..24,
            },
            register Baudrate {
                type RWCapability = RW;
                const ADDRESS: usize = 42;
                const SIZE_BYTES: usize = 2;

                value: u16 = 0..16,
            }
        }
    );
}

#[derive(Debug, num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
#[repr(u16)]
pub enum Manufacturer {
    MegaCorpX,
    Indy,
}

fn main() {
    // let mut test_device = TestDevice::new();

    // println!("{:?}", test_device.id().read().unwrap());

    // test_device.baudrate().write(|w| w.value(12)).unwrap();

    // write_baud(&mut test_device);
    // assert_eq!(test_device.baudrate().read().unwrap().value().unwrap(), 12);
}

// #[inline(never)]
// fn write_baud<const SIZE: usize>(device: &mut TestDevice<SIZE>) {
//     device.baudrate().write(|w| w.value(12)).unwrap();
// }
