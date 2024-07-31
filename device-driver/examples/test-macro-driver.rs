use std::ops::Range;

use bitvec::array::BitArray;
use device_driver::{
    embedded_io::{Read, Write},
    AddressableDevice, AsyncBufferDevice, AsyncCommandDevice, AsyncRegisterDevice, BufferDevice,
    CommandDevice, Register, RegisterDevice,
};

pub struct TestDevice {
    device_memory: [u8; 128],
    last_command: u32,
    last_buffer: u32,
}

impl AddressableDevice for TestDevice {
    type AddressType = u8;
}

impl RegisterDevice for TestDevice {
    type Error = ();

    fn write_register<const SIZE_BYTES: usize>(
        &mut self,
        address: Self::AddressType,
        data: &BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error> {
        self.device_memory[address as usize..][..SIZE_BYTES].copy_from_slice(data.as_raw_slice());

        Ok(())
    }

    fn read_register<const SIZE_BYTES: usize>(
        &mut self,
        address: Self::AddressType,
        data: &mut BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error> {
        data.as_raw_mut_slice()
            .copy_from_slice(&self.device_memory[address as usize..][..SIZE_BYTES]);
        Ok(())
    }
}

impl AsyncRegisterDevice for TestDevice {
    type Error = ();

    async fn write_register<const SIZE_BYTES: usize>(
        &mut self,
        address: Self::AddressType,
        data: &BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error> {
        self.device_memory[address as usize..][..SIZE_BYTES].copy_from_slice(data.as_raw_slice());

        Ok(())
    }

    async fn read_register<const SIZE_BYTES: usize>(
        &mut self,
        address: Self::AddressType,
        data: &mut BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error> {
        data.as_raw_mut_slice()
            .copy_from_slice(&self.device_memory[address as usize..][..SIZE_BYTES]);
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
            /// Interrupt flags
            register IntFlags {
                type RWType = ReadOnly;
                const ADDRESS: u8 = 16;
                const SIZE_BITS: usize = 8;

                rx: bool = 0,
                tx: bool = 1,
            },
            /// Interrupt enable flags
            ref register IntEnable = IntFlags {
                type RWType = ReadWrite;
                const ADDRESS: u8 = 17;
            },
            /// Baudrate register
            register Baudrate {
                type RWType = RW;
                type ByteOrder = LE; // Everything is BE by default
                const ADDRESS: u8 = 42;
                const SIZE_BITS: usize = 16;

                /// Baudrate value
                value: u16 = 0..16,
            },
            /// Instance of the Foo block
            block Foo0 {
                const BASE_ADDRESS: u8 = 64;

                /// Baudrate register
                register Baudrate {
                    type RWType = RW;
                    type ByteOrder = LE; // Everything is BE by default
                    const ADDRESS: u8 = 0;
                    const SIZE_BITS: usize = 16;

                    /// Baudrate value
                    value: u16 = 0..16,
                },
            },
            /// Second instance of the Foo block
            ref block Foo1 = Foo0 {
                const BASE_ADDRESS: u8 = 80;
            },
            /// Instance of the Bar block
            block Bar {
                const BASE_ADDRESS: u8 = 96;
                const REPEAT = {
                    count: 3,
                    stride: 2,
                };

                /// Baudrate register
                register Baudrate {
                    type RWType = RW;
                    type ByteOrder = LE; // Everything is BE by default
                    const ADDRESS: u8 = 0;
                    const SIZE_BITS: usize = 16;

                    /// Baudrate value
                    value: u16 = 0..16,
                },
            },
            register Foo {
                type RWType = RW;
                const ADDRESS: u8 = 0;
                const SIZE_BITS: usize = 16;
                const RESET_VALUE: u16 = 0x1234;

                #[cfg(windows)]
                value: u16 = 0..16,
                #[cfg(not(windows))]
                value: u16 = 0..16,
            },
            #[cfg(windows)]
            register CfgReg {
                type RWType = RW;
                const ADDRESS: u8 = 0;
                const SIZE_BITS: usize = 16;
                const RESET_VALUE: u16 = 0x1234;

                value: u16 = 0..16,
            },
            #[cfg(not(windows))]
            register CfgReg {
                type RWType = RW;
                const ADDRESS: u8 = 0;
                const SIZE_BITS: usize = 16;
                const RESET_VALUE: u16 = 0x5678;

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

    assert!(!test_device.int_flags().read().unwrap().rx());
    test_device.int_enable().write(|w| w.rx(true)).unwrap();
    assert!(test_device.int_enable().read().unwrap().rx());
    assert_eq!(test_device.device_memory[17], 1);

    test_device.baudrate().write(|w| w.value(12)).unwrap();

    write_baud(&mut test_device);
    assert_eq!(test_device.baudrate().read().unwrap().value(), 12);
    // Test it's actually doing little endian
    assert_eq!(test_device.device_memory[42], 12);
    assert_eq!(test_device.device_memory[43], 0);

    test_device
        .foo_0()
        .baudrate()
        .write(|w| w.value(12))
        .unwrap();
    assert_eq!(test_device.foo_0().baudrate().read().unwrap().value(), 12);
    assert_eq!(test_device.device_memory[64], 12);
    assert_eq!(test_device.device_memory[65], 0);

    test_device
        .foo_1()
        .baudrate()
        .write(|w| w.value(34))
        .unwrap();
    assert_eq!(test_device.foo_1().baudrate().read().unwrap().value(), 34);
    assert_eq!(test_device.device_memory[80], 34);
    assert_eq!(test_device.device_memory[81], 0);

    test_device.foo().clear().unwrap();
    assert_eq!(test_device.foo().read().unwrap().value(), 0x1234);

    test_device.foo().write(|w| w.value(5)).unwrap();
    assert_eq!(test_device.foo().read().unwrap().value(), 5);

    test_device.foo().write_with_zero(|w| w).unwrap();
    assert_eq!(test_device.foo().read().unwrap().value(), 0);

    test_device.foo().write(|w| w).unwrap();
    assert_eq!(test_device.foo().read().unwrap().value(), 0x1234);

    test_device.cfg_reg().clear().unwrap();
    #[cfg(windows)]
    assert_eq!(test_device.cfg_reg().read().unwrap().value(), 0x1234);
    #[cfg(not(windows))]
    assert_eq!(test_device.cfg_reg().read().unwrap().value(), 0x5678);

    test_device.sleep().dispatch().unwrap();
    assert_eq!(test_device.last_command, 0);

    test_device.burn().dispatch().unwrap();
    assert_eq!(test_device.last_command, 0xDEAD);

    let mut data = [1, 2, 3, 4, 5];
    test_device.fifo().write_all(&data).unwrap();
    test_device.fifo().read_exact(&mut data).unwrap();
    assert_eq!(data, [1, 2, 3, 4, 5]);
    assert_eq!(test_device.last_buffer, 124);

    // Create a mask
    let register_mask = registers::Baudrate::ZERO
        .into_w()
        .value(1234)
        .into_register();

    // Write test value
    test_device.baudrate().write(|w| w.value(u16::MAX)).unwrap();

    // Read back and apply mask
    let read_baud = (*test_device.baudrate().read().unwrap() & register_mask).into_r();
    assert_eq!(read_baud.value(), 1234);
}

#[inline(never)]
fn write_baud(device: &mut TestDevice) {
    device.baudrate().write(|w| w.value(12)).unwrap();
}
