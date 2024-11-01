use device_driver::RegisterInterface;

pub struct DeviceInterface {
    device_memory: [u8; 128],
}

impl Default for DeviceInterface {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceInterface {
    pub const fn new() -> Self {
        Self {
            device_memory: [0; 128],
        }
    }
}

impl RegisterInterface for DeviceInterface {
    type Error = ();
    type AddressType = u8;

    fn write_register(
        &mut self,
        address: Self::AddressType,
        _size_bits: u32,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        self.device_memory[address as usize..][..data.len()].copy_from_slice(data);

        Ok(())
    }

    fn read_register(
        &mut self,
        address: Self::AddressType,
        _size_bits: u32,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
        data.copy_from_slice(&self.device_memory[address as usize..][..data.len()]);
        Ok(())
    }
}

device_driver::create_device!(
    device_name: MyTestDevice,
    dsl: {
        config {
            type RegisterAddressType = u8;
        }
        #[cfg(not(windows))]
        register Foo {
            const ADDRESS = 0;
            const SIZE_BITS = 8;
            const RESET_VALUE = 20;

            value: bool = 0,
            generated: uint as try enum Gen {
                A = 0,
            } = 1..2,
        },
        #[cfg(windows)]
        register Foo {
            const ADDRESS = 1;
            const SIZE_BITS = 8;
            const RESET_VALUE = 10;

            value: bool = 0,
            generated: uint as try enum Gen {
                A = 1,
            } = 1..2,
        },
        /// This ref uses whatever Foo ends up existing. But one must exist!
        ref FooRef = register Foo {
            const ADDRESS = 2;
        }
    }
);

#[test]
fn test_basic_read_modify_write() {
    let mut device = MyTestDevice::new(DeviceInterface::new());
    device
        .foo()
        .write_with_zero(|reg| reg.set_value(true))
        .unwrap();

    #[cfg(not(windows))]
    assert_eq!(device.interface().device_memory[0], 0x01);
    #[cfg(windows)]
    assert_eq!(device.interface().device_memory[1], 0x01);

    #[cfg(not(windows))]
    assert_eq!(Gen::A as u8, 0);
    #[cfg(windows)]
    assert_eq!(Gen::A as u8, 1);
}
