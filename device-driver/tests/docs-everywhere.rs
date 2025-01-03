//! Test for making sure everything is documented

#![deny(missing_docs)]

device_driver::create_device!(
    device_name: MyTestDevice,
    dsl: {
        config {
            type RegisterAddressType = u8;
            type CommandAddressType = u8;
        }
        register Foo {
            const ADDRESS = 0;
            const SIZE_BITS = 8;

            value0: bool = 0,
        },
        command Bar {
            const ADDRESS = 0;
            const SIZE_BITS_IN = 8;
            const SIZE_BITS_OUT = 8;

            in {
                value0: bool = 0,
            }
            out {
                value0: bool = 0,
            }
        },
    }
);
