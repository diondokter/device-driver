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
        register FooRepeated {
            const ADDRESS = 3;
            const SIZE_BITS = 8;
            const REPEAT = {
                count: usize as enum Blah {
                    A, B, C = 5, D = 4,
                },
                stride: 3,
            };

            value: uint = 0..8,
        },
        ref FooRef = register FooRepeated {
            const ADDRESS = 4; // Note: Address overlap checking is very limited when using repeat count
        },
        ref FooRefExtra = register FooRepeated {
            const ADDRESS = 5;
            const REPEAT = {
                count: usize as CustomIndex,
                stride: 3,
            };
        }
    }
);

pub enum CustomIndex {
    A,
    B,
}

impl From<CustomIndex> for usize {
    fn from(value: CustomIndex) -> Self {
        match value {
            CustomIndex::A => 0,
            CustomIndex::B => 1,
        }
    }
}

#[test]
fn test_repeated_read_modify_write() {
    let mut device = MyTestDevice::new(DeviceInterface::new());
    device
        .foo_repeated(Blah::C)
        .modify(|reg| {
            reg.set_value(12);
        })
        .unwrap();

    device
        .foo_ref_extra(CustomIndex::B)
        .modify(|reg| {
            reg.set_value(42);
        })
        .unwrap();

    assert_eq!(device.interface.device_memory[18], 12);
    assert_eq!(device.interface.device_memory[8], 42);
}
