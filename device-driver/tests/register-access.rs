//! Having registers and fields with different access specifiers should still compile

device_driver::create_device!(
    ddsl: "
        device MyTestDevice {
            register-address-type u8

            register Foo {
                address 0
                fields size-bits=8 {
                    /// X
                    (bool)value0 @0
                }
            }
            register Bar {
                access RO
                address 1
                fields size-bits=8 {
                    /// X
                    (bool)value0 WO @0
                }
            }
            register Baz {
                access WO
                address 2
                fields size-bits=8 {
                    /// X
                    (bool)value0 @0
                }
            }
        }
    "
);
