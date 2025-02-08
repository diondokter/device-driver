device_driver::create_device!(
    device_name: MyTestDevice,
    dsl: {
        config {
            type RegisterAddressType = u8;
            type DefaultByteOrder = LE;
        }
        register Foo {
            const ADDRESS = 0;
            const SIZE_BITS = 32;
            const RESET_VALUE = 0xFFFFFFFF;

            value: uint = 0..32,
        },
    }
);

#[cfg(test)]
mod tests {
    use super::field_sets::Foo;

    #[test]
    fn and() {
        assert_eq!(Foo::new() & Foo::new_zero(), Foo::new_zero());

        let mut test_foo = Foo::new_zero();
        test_foo.set_value(0x12345678);

        assert_eq!(Foo::new() & test_foo, test_foo);
        assert_eq!(test_foo & Foo::new_zero(), Foo::new_zero());

        test_foo &= Foo::new_zero();

        assert_eq!(test_foo, Foo::new_zero());
    }

    #[test]
    fn or() {
        assert_eq!(Foo::new_zero() | Foo::new(), Foo::new());

        let mut test_foo = Foo::new_zero();
        test_foo.set_value(0x12345678);

        assert_eq!(Foo::new_zero() | test_foo, test_foo);
        assert_eq!(test_foo | Foo::new(), Foo::new());

        test_foo |= Foo::new();

        assert_eq!(test_foo, Foo::new());
    }

    #[test]
    fn xor() {
        assert_eq!(Foo::new_zero() ^ Foo::new(), Foo::new());
        assert_eq!(Foo::new_zero() ^ Foo::new() ^ Foo::new(), Foo::new_zero());

        let mut test_foo = Foo::new_zero();
        test_foo.set_value(0x12345678);

        assert_eq!(Foo::new_zero() ^ test_foo, test_foo);
        assert_eq!(test_foo ^ Foo::new(), !test_foo);

        test_foo ^= test_foo;

        assert_eq!(test_foo, Foo::new_zero());
    }

    #[test]
    fn not() {
        assert_eq!(!Foo::new_zero(), Foo::new());
        assert_eq!(!Foo::new(), Foo::new_zero());
    }
}
