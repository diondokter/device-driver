use device_driver::{BufferInterface, BufferInterfaceError};

pub struct DeviceInterface {
    last_address: u8,
    last_val: Vec<u8>,
}

#[derive(Debug)]
pub enum Error {}

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        todo!()
    }
}

impl BufferInterfaceError for DeviceInterface {
    type Error = Error;
}

impl BufferInterface for DeviceInterface {
    type AddressType = u8;

    fn write(&mut self, address: Self::AddressType, buf: &[u8]) -> Result<usize, Self::Error> {
        self.last_address = address;
        self.last_val = buf.to_vec();
        Ok(buf.len())
    }

    fn flush(&mut self, address: Self::AddressType) -> Result<(), Self::Error> {
        self.last_address = address;
        Ok(())
    }

    fn read(&mut self, address: Self::AddressType, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.last_address = address;
        let len = self.last_val.len().min(buf.len());
        buf[..len].copy_from_slice(&self.last_val[..len]);
        Ok(len)
    }
}

device_driver::create_device!(
    kdl: "
        device MyTestDevice {
            default-byte-order LE
            buffer-address-type u8

            /// A read only buffer
            buffer RoBuf {
                access RO
                address 0
            }
            buffer WoBuf {
                access WO
                address 1
            }
        }
    "
);

#[test]
fn buffer_write_read() {
    let mut device = MyTestDevice::new(DeviceInterface {
        last_address: 0xFF,
        last_val: Vec::new(),
    });

    device.wo_buf().write(&[0, 1, 2, 3]).unwrap();
    assert_eq!(device.interface.last_address, 1);

    let mut buffer = [0; 8];
    let len = device.ro_buf().read(&mut buffer).unwrap();
    assert_eq!(device.interface.last_address, 0);
    assert_eq!(len, 4);
    assert_eq!(&buffer[..len], &[0, 1, 2, 3]);
}

#[test]
fn impls_embedded_io() {
    let mut device = MyTestDevice::new(DeviceInterface {
        last_address: 0xFF,
        last_val: Vec::new(),
    });

    fn is_read(_: impl embedded_io::Read) {}
    fn is_write(_: impl embedded_io::Write) {}

    is_read(device.ro_buf());
    is_write(device.wo_buf());
}
