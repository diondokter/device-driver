//! Test for making sure everything is documented

#![deny(missing_docs)]

device_driver::compile!(
    ddsl: "
        device MyTestDevice {
            register-address-type: u8,
            command-address-type: u8,
            buffer-address-type: u8,
            /// X
            register Foo {
                address: 0,
                fields:
                    /// X
                    fieldset FooFields {
                        size-bits: 8,
                        /// X
                        field value0 0 -> bool
                    }
            },
            /// X
            command Bar {
                address: 0,
                fields-in:
                    /// X
                    fieldset BarFieldsIn {
                        size-bits: 8,
                        /// X
                        field value0 0 -> bool
                    },
                fields-out:
                    /// X
                    fieldset BarFieldsOut {
                        size-bits: 8,
                        /// X
                        field value0 0 -> bool
                    },
            },
            /// X
            buffer Baz {
                address: 0
            },
            /// X
            block B {
                address-offset: 0
            }
        }
    "
);
