use device_driver::{FieldsetMetadata, RegisterInterface};

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
        _metadata: &FieldsetMetadata,
        address: Self::AddressType,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        assert_eq!(data.len(), 3);
        self.device_memory[address as usize..][..data.len()].copy_from_slice(data);

        Ok(())
    }

    fn read_register(
        &mut self,
        _metadata: &FieldsetMetadata,
        address: Self::AddressType,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
        assert_eq!(data.len(), 3);
        data.copy_from_slice(&self.device_memory[address as usize..][..data.len()]);
        Ok(())
    }
}

device_driver::compile!(
    options: [
        "defmt-feature=defmt"
    ],
    ddsl: "
        device MyTestDevice {
            byte-order: LE,
            register-address-type: u8,

            /// This is the Foo register
            register Foo {
                address: 0,
                fields: fieldset FooFields {
                    size-bytes: 3,

                    /// This is a bool!
                    field value0 0 -> bool,
                    field value1 15:1 -> uint,
                    field value2 23:16 -> int,
                }
            },
            /// This is the Foo register
            register FooRepeated[4*3] {
                address: 3,
                fields: FooFields,
            }
        }
    "
);

#[test]
fn test_basic_read_modify_write() {
    let mut device = MyTestDevice::new(DeviceInterface::new());

    device.foo().write(|reg| reg.set_value_1(12345)).unwrap();
    let reg = device.foo().read().unwrap();

    assert!(!reg.value_0());
    assert_eq!(reg.value_1(), 12345);
    assert_eq!(reg.value_2(), 0i8);

    device
        .foo()
        .modify(|reg| {
            reg.set_value_0(true);
            reg.set_value_2(-1);
        })
        .unwrap();

    let reg = device.foo().read().unwrap();

    assert!(reg.value_0());
    assert_eq!(reg.value_1(), 12345);
    assert_eq!(reg.value_2(), -1);

    assert_eq!(
        u32::from_le_bytes(*device.interface.device_memory[0..4].as_array().unwrap()),
        1 | 12345 << 1 | ((-1i8 as u8) as u32) << 16
    );
}

#[test]
#[should_panic]
fn test_repeated_too_large_index() {
    let mut device = MyTestDevice::new(DeviceInterface::new());
    device.foo_repeated().plan_at(4);
}

#[test]
fn test_repeated_read_modify_write() {
    let mut device = MyTestDevice::new(DeviceInterface::new());
    device
        .foo_repeated()
        .modify_at(2, |reg| {
            reg.set_value_0(true);
            reg.set_value_1(12345);
            reg.set_value_2(-1);
        })
        .unwrap();

    assert_eq!(
        &device.interface.device_memory[9..12],
        &[(0x39 << 1) + 1, 0x30 << 1, 0xFF]
    );
}
