use device_driver::RegisterInterface;

device_driver::create_device!(
    kdl: "
        device MyTestDevice {
            default-byte-order LE
            register-address-type u8

            register Foo {
                address 0
                reset-value 0xFFFFFFFF

                fields size-bits=32 {
                    (uint)value @31:0
                }
            }
        }
    "
);

pub struct DeviceInterface;

impl RegisterInterface for DeviceInterface {
    type Error = ();
    type AddressType = u8;

    fn write_register(
        &mut self,
        _address: Self::AddressType,
        _size_bits: u32,
        _data: &[u8],
    ) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn read_register(
        &mut self,
        _address: Self::AddressType,
        _size_bits: u32,
        _data: &mut [u8],
    ) -> Result<(), Self::Error> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn and() {
        let reset_foo = MyTestDevice::new(DeviceInterface).foo().reset_value();

        assert_eq!(reset_foo & FooFieldSet::new(), FooFieldSet::new());

        let mut test_foo = FooFieldSet::new();
        test_foo.set_value(0x12345678);

        assert_eq!(reset_foo & test_foo, test_foo);
        assert_eq!(test_foo & FooFieldSet::new(), FooFieldSet::new());

        test_foo &= FooFieldSet::new();

        assert_eq!(test_foo, FooFieldSet::new());
    }

    #[test]
    fn or() {
        let reset_foo = MyTestDevice::new(DeviceInterface).foo().reset_value();

        assert_eq!(FooFieldSet::new() | reset_foo, reset_foo);

        let mut test_foo = FooFieldSet::new();
        test_foo.set_value(0x12345678);

        assert_eq!(FooFieldSet::new() | test_foo, test_foo);
        assert_eq!(test_foo | reset_foo, reset_foo);

        test_foo |= reset_foo;

        assert_eq!(test_foo, reset_foo);
    }

    #[test]
    fn xor() {
        let reset_foo = MyTestDevice::new(DeviceInterface).foo().reset_value();

        assert_eq!(FooFieldSet::new() ^ reset_foo, reset_foo);
        assert_eq!(
            FooFieldSet::new() ^ reset_foo ^ reset_foo,
            FooFieldSet::new()
        );

        let mut test_foo = FooFieldSet::new();
        test_foo.set_value(0x12345678);

        assert_eq!(FooFieldSet::new() ^ test_foo, test_foo);
        assert_eq!(test_foo ^ reset_foo, !test_foo);

        test_foo ^= test_foo;

        assert_eq!(test_foo, FooFieldSet::new());
    }

    #[test]
    fn not() {
        let reset_foo = MyTestDevice::new(DeviceInterface).foo().reset_value();

        assert_eq!(!FooFieldSet::new(), reset_foo);
        assert_eq!(!reset_foo, FooFieldSet::new());
    }
}
