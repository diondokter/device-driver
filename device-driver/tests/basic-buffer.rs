use device_driver::{BufferInterface, BufferInterfaceError};

pub struct DeviceInterface {
    last_address: u8,
    last_val: Vec<u8>,
}

impl BufferInterfaceError for DeviceInterface {
    type Error = ();
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

device_driver_macros::implement_device!(
    device_name: MyTestDevice,
    dsl: {
        config {
            type BufferAddressType = u8;
            type DefaultByteOrder = LE;
        }
        /// A read only buffer
        buffer RoBuf: RO = 0,
        buffer WoBuf: WO = 1,
    }
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
