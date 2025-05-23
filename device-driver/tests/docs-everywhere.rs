//! Test for making sure everything is documented

#![deny(missing_docs)]

device_driver::create_device!(
    device_name: MyTestDevice,
    dsl: {
        config {
            type RegisterAddressType = u8;
            type CommandAddressType = u8;
            type BufferAddressType = u8;
        }
        /// X
        register Foo {
            const ADDRESS = 0;
            const SIZE_BITS = 8;

            /// X
            value0: bool = 0,
        },
        /// X
        command Bar {
            const ADDRESS = 0;
            const SIZE_BITS_IN = 8;
            const SIZE_BITS_OUT = 8;

            in {
                /// X
                value0: bool = 0,
            }
            out {
                /// X
                value0: bool = 0,
            }
        },
        /// X
        buffer Baz = 0,
        /// X
        block B {

        },
        /// X
        ref R = register Foo {
            const ADDRESS = 1;
        }
    }
);
