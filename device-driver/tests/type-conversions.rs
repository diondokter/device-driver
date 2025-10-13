use device_driver::{ConversionError, RegisterInterface};

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
                fields size-bits=64 {
                    /// Try needed since [MyTryEnum] doesn't impl [From]
                    (uint:MyTryEnum?)convert_custom_try @1:0
                    /// No try needed since [MyEnum] impls [From]
                    (uint:MyEnum)convert_custom @3:2
                    /// Try needed since not all bit patters are covered
                    (uint:GenTryEnum?)convert_generated_try @5:4 {
                        A
                        B
                    }
                    /// No try needed because it covers every bit pattern (2 bit)
                    (uint:GenEnum)convert_generated @7:6 {
                        A
                        B
                        C 2
                        D
                    }
                    /// No try needed since default
                    (uint:GenDefaultEnum)convert_generated_default @9:8 {
                        A
                        B default
                    }
                    /// No try needed since catch-all
                    (uint:GenCatchAllEnum)convert_generated_catchall @11:10 {
                        A
                        B catch-all
                    }
                    /// No try needed since it recognizes GenEnum (even though it doesn implement From<u8>)
                    (uint:GenEnum)convert_generated_copied @13:12
                    /// Try needed since it recognizes GenEnum, but the bits are too big (3 bit vs 2 bit)
                    (uint:GenEnum?)convert_generated_copied_too_large @16:14
                }
            }
            enum MyTryEnum {
                A
                B
                C
            }
            enum MyEnum {
                A
                B
                C
                D
            }
        }
    "
);

#[test]
fn test_basic_read_modify_write() {
    let mut device = MyTestDevice::new(DeviceInterface::new());

    device
        .foo()
        .write(|reg| {
            reg.set_convert_custom_try(MyTryEnum::C);
            reg.set_convert_custom(MyEnum::D);
            reg.set_convert_generated_try(GenTryEnum::B);
            reg.set_convert_generated(GenEnum::C);
            reg.set_convert_generated_default(GenDefaultEnum::B);
            reg.set_convert_generated_catchall(GenCatchAllEnum::B(3));
        })
        .unwrap();

    assert_eq!(
        device.foo().read().unwrap().convert_custom_try(),
        Result::<_, ConversionError<u8>>::Ok(MyTryEnum::C)
    );
    assert_eq!(device.foo().read().unwrap().convert_custom(), MyEnum::D);
    assert_eq!(
        device.foo().read().unwrap().convert_generated_try(),
        Result::<_, ConversionError<u8>>::Ok(GenTryEnum::B)
    );
    assert_eq!(device.foo().read().unwrap().convert_generated(), GenEnum::C);
    assert_eq!(
        device.foo().read().unwrap().convert_generated_default(),
        GenDefaultEnum::B
    );
    assert_eq!(
        device.foo().read().unwrap().convert_generated_catchall(),
        GenCatchAllEnum::B(3)
    );
}
