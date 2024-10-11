use device_driver::{ConversionError, RegisterInterface};

pub struct DeviceInterface {
    device_memory: [u8; 128],
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
            type DefaultByteOrder = LE;
        }
        register Foo {
            const ADDRESS = 0;
            const SIZE_BITS = 64;

            /// Try needed since [my_mod::MyTryEnum] doesn't impl [From]
            convert_custom_try: uint as try my_mod::MyTryEnum = 0..2,
            /// No try needed since [MyEnum] impls [From]
            convert_custom: uint as MyEnum = 2..4,
            /// Try needed since not all bit patters are covered
            convert_generated_try: uint as try enum GenTryEnum {
                A,
                B,
            } = 4..6,
            /// No try needed because it covers every bit pattern (2 bit)
            convert_generated: uint as enum GenEnum {
                A,
                B,
                #[cfg(windows)]
                C = 2,
                #[cfg(unix)]
                C = 2,
                D
            } = 6..8,
            /// No try needed since default
            convert_generated_default: uint as enum GenDefaultEnum {
                A,
                B = default,
            } = 8..10,
            /// No try needed since catch-all
            convert_generated_catchall: uint as enum GenCatchAllEnum {
                A,
                B = catch_all,
            } = 10..12,
            /// No try needed since it recognizes GenEnum (even though it doesn implement From<u8>)
            convert_generated_copied: uint as GenEnum = 12..14,
            /// Try needed since it recognizes GenEnum, but the bits are too big (3 bit vs 2 bit)
            convert_generated_copied_too_large: uint as try GenEnum = 14..17,
        },
    }
);

pub mod my_mod {
    #[derive(Debug, PartialEq, Eq)]
    #[repr(u8)]
    pub enum MyTryEnum {
        A,
        B,
        C,
    }

    impl TryFrom<u8> for MyTryEnum {
        type Error = u8;

        fn try_from(value: u8) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(MyTryEnum::A),
                1 => Ok(MyTryEnum::B),
                2 => Ok(MyTryEnum::C),
                val => Err(val),
            }
        }
    }

    impl From<MyTryEnum> for u8 {
        fn from(value: MyTryEnum) -> Self {
            value as _
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MyEnum {
    A,
    B,
    C,
    D,
}

impl From<u8> for MyEnum {
    fn from(value: u8) -> Self {
        match value {
            0 => MyEnum::A,
            1 => MyEnum::B,
            2 => MyEnum::C,
            3 => MyEnum::D,
            _ => panic!(),
        }
    }
}

impl From<MyEnum> for u8 {
    fn from(value: MyEnum) -> Self {
        value as _
    }
}

#[test]
fn test_basic_read_modify_write() {
    let mut device = MyTestDevice::new(DeviceInterface::new());

    device
        .foo()
        .write(|reg| {
            reg.set_convert_custom_try(my_mod::MyTryEnum::C);
            reg.set_convert_custom(MyEnum::D);
            reg.set_convert_generated_try(GenTryEnum::B);
            reg.set_convert_generated(GenEnum::C);
            reg.set_convert_generated_default(GenDefaultEnum::B);
            reg.set_convert_generated_catchall(GenCatchAllEnum::B(3));
        })
        .unwrap();

    assert_eq!(
        device.foo().read().unwrap().convert_custom_try(),
        Result::<_, u8>::Ok(my_mod::MyTryEnum::C)
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
