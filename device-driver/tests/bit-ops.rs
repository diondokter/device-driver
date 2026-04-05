use device_driver::{FieldsetMetadata, RegisterInterface};

device_driver::compile!(
    ddsl: "
        device MyTestDevice {
            byte-order: LE,
            register-address-type: u8,

            register Foo {
                address: 0,
                reset: 0xFFFFFFFF,

                fields: fieldset FooFieldSet {
                    size-bytes: 4,

                    field value 31:0 -> uint
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
        _metadata: &FieldsetMetadata,
        _address: Self::AddressType,
        _data: &[u8],
    ) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn read_register(
        &mut self,
        _metadata: &FieldsetMetadata,
        _address: Self::AddressType,
        _data: &mut [u8],
    ) -> Result<(), Self::Error> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use device_driver::Fieldset;

    use super::*;

    #[test]
    fn and() {
        let reset_foo = MyTestDevice::new(DeviceInterface).foo().reset_value();

        assert_eq!(reset_foo & FooFieldSet::ZERO, FooFieldSet::ZERO);

        let mut test_foo = FooFieldSet::ZERO;
        test_foo.set_value(0x12345678);

        assert_eq!(reset_foo & test_foo, test_foo);
        assert_eq!(test_foo & FooFieldSet::ZERO, FooFieldSet::ZERO);

        test_foo &= FooFieldSet::ZERO;

        assert_eq!(test_foo, FooFieldSet::ZERO);
    }

    #[test]
    fn or() {
        let reset_foo = MyTestDevice::new(DeviceInterface).foo().reset_value();

        assert_eq!(FooFieldSet::ZERO | reset_foo, reset_foo);

        let mut test_foo = FooFieldSet::ZERO;
        test_foo.set_value(0x12345678);

        assert_eq!(FooFieldSet::ZERO | test_foo, test_foo);
        assert_eq!(test_foo | reset_foo, reset_foo);

        test_foo |= reset_foo;

        assert_eq!(test_foo, reset_foo);
    }

    #[test]
    fn xor() {
        let reset_foo = MyTestDevice::new(DeviceInterface).foo().reset_value();

        assert_eq!(FooFieldSet::ZERO ^ reset_foo, reset_foo);
        assert_eq!(
            FooFieldSet::ZERO ^ reset_foo ^ reset_foo,
            FooFieldSet::ZERO
        );

        let mut test_foo = FooFieldSet::ZERO;
        test_foo.set_value(0x12345678);

        assert_eq!(FooFieldSet::ZERO ^ test_foo, test_foo);
        assert_eq!(test_foo ^ reset_foo, !test_foo);

        test_foo ^= test_foo;

        assert_eq!(test_foo, FooFieldSet::ZERO);
    }

    #[test]
    fn not() {
        let reset_foo = MyTestDevice::new(DeviceInterface).foo().reset_value();

        assert_eq!(!FooFieldSet::ZERO, reset_foo);
        assert_eq!(!reset_foo, FooFieldSet::ZERO);
    }
}
