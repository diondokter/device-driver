//! Having registers and fields with different access specifiers should still compile

device_driver::create_device!(
    ddsl: "
        device MyTestDevice {
            register-address-type: u8,

            register Foo {
                address: 0,
                fields: fieldset FooFields {
                    size-bits: 8,

                    /// X
                    field value0 0 -> bool,
                }
            },
            register Bar {
                access: RO,
                address: 1,
                fields: fieldset BarFields {
                    size-bits: 8,

                    /// X
                    field value0 WO 0 -> bool,
                }
            },
            register Baz {
                access: WO,
                address: 2,
                fields: fieldset BazFields {
                    size-bits: 8,

                    /// X
                    field value0 0 -> bool,
                }
            }
        }
    "
);
