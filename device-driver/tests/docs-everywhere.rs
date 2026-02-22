//! Test for making sure everything is documented

#![deny(missing_docs)]

device_driver::create_device!(
    ddsl: "
        device MyTestDevice {
            register-address-type u8
            command-address-type u8
            buffer-address-type u8
            /// X
            register Foo {
                address 0
                /// X
                fields size-bits=8 {
                    /// X
                    (bool)value0 @0
                }
            }
            /// X
            command Bar {
                address 0
                /// X
                in size-bits=8 {
                    /// X
                    (bool)value0 @0
                }
                /// X
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
    "
);
