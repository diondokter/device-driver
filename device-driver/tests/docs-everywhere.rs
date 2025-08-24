//! Test for making sure everything is documented

#![deny(missing_docs)]

device_driver::create_device!(
    kdl: {
        device MyTestDevice {
            register-address-type u8
            command-address-type u8
            buffer-address-type u8
            /// X
            register Foo {
                address 0
                fields size-bits=8 {
                    /// X
                    (bool)value0 @0
                }
            }
            /// X
            command Bar {
                address 0
                in size-bits=8 {
                    /// X
                    (bool)value0 @0
                }
                out size-bits=8 {
                    /// X
                    (bool)value0 @0
                }
            }
            /// X
            buffer Baz {
                address 0
            }
            /// X
            block B {
                offset 0
            }
        }
    }
);
