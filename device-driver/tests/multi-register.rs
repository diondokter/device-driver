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
    kdl: "
        device MyTestDevice {
            byte-order LE
            register-address-type u8

            register Foo {
                address 0
                reset-value 0xFFFFFF
                fields size-bits=24 {
                    (bool)value0 @0
                    (uint)value1 @15:1
                    (int)value2 @23:17
                }
            }

            register Bar {
                address 3
                repeat count=8 stride=1
                fields size-bits=8 {
                    (int)value @7:0
                }
            }
        }
    "
);

#[test]
fn multi_test() {
    use device_driver::Block;
    let mut device = MyTestDevice::new(DeviceInterface::new());

    device
        .multi_write()
        .with(|d| d.bar().plan_with_zero_at(0))
        .with(|d| d.foo().plan())
        .execute(|(bar, _foo)| {
            bar.set_value(42);
        })
        .unwrap();

    let multi = device
        .multi_read()
        // TODO: Allow reading multiple. This would return a [Bar;N] that can also impl FieldSet.
        // Maybe N is a const generic on foo_repeated with default 1?
        // We'll also need a check whether it's allowed by the device rules. That's probably an assert.
        .with(|d| d.bar().plan_at(0))
        .with(|d| d.foo().plan())
        .execute()
        .unwrap();

    assert_eq!(
        multi,
        (
            BarFieldSet::from([42]),
            FooFieldSet::from([0xFF, 0xFF, 0xFF])
        )
    );

    device
        .multi_modify()
        .with(|d| d.bar().plan_at(0))
        .with(|d| d.foo().plan())
        .execute(|(bar, foo)| {
            bar.set_value(-5);
            foo.set_value_0(false);
        })
        .unwrap();

    let multi = device
        .multi_read()
        .with(|d| d.bar().plan_at(0))
        .with(|d| d.foo().plan())
        .execute()
        .unwrap();

    assert_eq!(
        multi,
        (
            BarFieldSet::from([-5i8 as u8]),
            FooFieldSet::from([0xFE, 0xFF, 0xFF])
        )
    );
}
