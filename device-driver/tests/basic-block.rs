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
    ddsl: "
        device MyTestDevice {
            byte-order LE
            register-address-type u8
            /// Block description
            block Bar {
                offset 10
                repeat count=2 stride=20
                /// This is the Foo register
                register Foo {
                    address 0
                    fields size-bits=24 {
                        /// This is a bool!
                        (bool)value0 @0
                        (uint)value1 @15:1
                        (int)value2 @23:16
                    }
                }
            }
        }
    "
);

#[test]
fn test_basic_read_modify_write() {
    let mut device = MyTestDevice::new(DeviceInterface::new());

    let mut bar = device.bar(1);

    bar.foo().write(|reg| reg.set_value_1(12345)).unwrap();
    let reg = bar.foo().read().unwrap();

    assert!(!reg.value_0());
    assert_eq!(reg.value_1(), 12345);
    assert_eq!(reg.value_2(), 0i8);

    bar.foo()
        .modify(|reg| {
            reg.set_value_0(true);
            reg.set_value_2(-1);
        })
        .unwrap();

    let reg = bar.foo().read().unwrap();

    assert!(reg.value_0());
    assert_eq!(reg.value_1(), 12345);
    assert_eq!(reg.value_2(), -1);

    assert_eq!(
        &device.interface.device_memory[30..33],
        &[(0x39 << 1) + 1, 0x30 << 1, 0xFF]
    );
}

#[test]
#[should_panic]
fn test_repeated_too_large_index() {
    let mut device = MyTestDevice::new(DeviceInterface::new());
    device.bar(2);
}
