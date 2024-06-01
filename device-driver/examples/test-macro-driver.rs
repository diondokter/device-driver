use std::ops::Range;

use bitvec::array::BitArray;
use device_driver::{
    embedded_io::{Read, Write},
    AsyncBufferDevice, AsyncCommandDevice, AsyncRegisterDevice, BufferDevice, CommandDevice,
    Register, RegisterDevice,
};

pub struct TestDevice {
    device_memory: [u8; 128],
    last_command: u32,
    last_buffer: u32,
}

impl RegisterDevice for TestDevice {
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

impl AsyncRegisterDevice for TestDevice {
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

impl CommandDevice for TestDevice {
    type Error = ();

    fn dispatch_command(&mut self, id: u32) -> Result<(), Self::Error> {
        self.last_command = id;

        Ok(())
    }
}

impl AsyncCommandDevice for TestDevice {
    type Error = ();

    async fn dispatch_command(&mut self, id: u32) -> Result<(), Self::Error> {
        self.last_command = id;

        Ok(())
    }
}

const BUFFER_RANGE: Range<usize> = 64..128;
impl BufferDevice for TestDevice {
    fn write(&mut self, id: u32, buf: &[u8]) -> Result<usize, embedded_io::ErrorKind> {
        self.last_buffer = id;

        if buf.len() > BUFFER_RANGE.len() {
            return Err(embedded_io::ErrorKind::InvalidData);
        }

        self.device_memory[BUFFER_RANGE][..buf.len()].copy_from_slice(buf);
        Ok(buf.len())
    }

    fn read(&mut self, id: u32, buf: &mut [u8]) -> Result<usize, embedded_io::ErrorKind> {
        self.last_buffer = id;

        let max = buf.len().min(BUFFER_RANGE.len());
        buf[..max].copy_from_slice(&self.device_memory[BUFFER_RANGE][..max]);

        Ok(max)
    }
}

impl AsyncBufferDevice for TestDevice {
    async fn write(&mut self, id: u32, buf: &[u8]) -> Result<usize, embedded_io::ErrorKind> {
        self.last_buffer = id;

        if buf.len() > BUFFER_RANGE.len() {
            return Err(embedded_io::ErrorKind::InvalidData);
        }

        self.device_memory[BUFFER_RANGE][..buf.len()].copy_from_slice(buf);
        Ok(buf.len())
    }

    async fn read(&mut self, id: u32, buf: &mut [u8]) -> Result<usize, embedded_io::ErrorKind> {
        self.last_buffer = id;

        let max = buf.len().min(BUFFER_RANGE.len());
        buf[..max].copy_from_slice(&self.device_memory[BUFFER_RANGE][..max]);

        Ok(max)
    }
}

impl Default for TestDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl TestDevice {
    pub fn new() -> Self {
        // Normally we'd take like a SPI here or something
        Self {
            device_memory: [0; 128],
            last_command: u32::MAX,
            last_buffer: u32::MAX,
        }
    }
}

pub mod registers {
    use super::*;

    device_driver_macros::implement_device!(
        impl TestDevice {
            register Id {
                type RWType = ReadOnly;
                const ADDRESS: u8 = 12;
                const SIZE_BITS: usize = 24;
                const RESET_VALUE: [u8] = [0, 0, 5];

                manufacturer: u16 as Manufacturer = 0..16,
                version: u8 = 16..20,
                edition: u8 as strict enum Edition {
                    One = 1,
                    Two,
                    /// Test!
                    Five = 5,
                    Rest = "default",
                } = 20..24,
            },
            /// Baudrate register
            register Baudrate {
                type RWType = RW;
                const ADDRESS: u8 = 42;
                const SIZE_BITS: usize = 16;

                /// Baudrate value
                value: u16 = 0..16,
            },
            register Foo {
                type RWType = RW;
                const ADDRESS: u8 = 0;
                const SIZE_BITS: usize = 16;
                const RESET_VALUE: u16 = 0x1234;

                value: u16 = 0..16,
            },
            command Sleep = 0,
            /// Let's out the magic smoke
            command Burn = 0xDEAD,
            buffer Terminal: RO = 123,
            /// A buffer you can write to and read from
            buffer Fifo: RW = 124,
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
    let mut test_device = TestDevice::new();

    println!("{:?}", test_device.id().read().unwrap());

    test_device.baudrate().write(|w| w.value(12)).unwrap();

    write_baud(&mut test_device);
    assert_eq!(test_device.baudrate().read().unwrap().value(), 12);

    test_device.foo().clear().unwrap();
    assert_eq!(test_device.foo().read().unwrap().value(), 0x1234);

    test_device.foo().write(|w| w.value(5)).unwrap();
    assert_eq!(test_device.foo().read().unwrap().value(), 5);

    test_device.foo().write_with_zero(|w| w).unwrap();
    assert_eq!(test_device.foo().read().unwrap().value(), 0);

    test_device.foo().write(|w| w).unwrap();
    assert_eq!(test_device.foo().read().unwrap().value(), 0x1234);

    test_device.sleep().dispatch().unwrap();
    assert_eq!(test_device.last_command, 0);

    test_device.burn().dispatch().unwrap();
    assert_eq!(test_device.last_command, 0xDEAD);

    let mut data = [1, 2, 3, 4, 5];
    test_device.fifo().write_all(&data).unwrap();
    test_device.fifo().read_exact(&mut data).unwrap();
    assert_eq!(data, [1, 2, 3, 4, 5]);
    assert_eq!(test_device.last_buffer, 124);
}

#[inline(never)]
fn write_baud(device: &mut TestDevice) {
    device.baudrate().write(|w| w.value(12)).unwrap();
}
