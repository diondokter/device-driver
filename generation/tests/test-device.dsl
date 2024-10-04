
config {
    type RegisterAddressType = u8;
    type DefaultByteOrder = LE;
}
block Bar {
    const ADDRESS_OFFSET = 10;
    const REPEAT = {
        count: 2,
        stride: 20,
    };

    ///This is the Foo register
    register Foo {
        const ADDRESS = 0;
        const SIZE_BITS = 24;

        ///This is a bool!
        value0: bool = 0..1,
        value1: uint = 1..16,
        value2: int = 16..24,
    },
},
///A command with inputs and outputs
command InOut {
    const ADDRESS = 3;
    const SIZE_BITS_IN = 16;
    const SIZE_BITS_OUT = 8;

    in {
        ///The value!
        val: uint = 0..16,
    }
    out {
        ///The value!
        val: uint = 0..8,
    }
},
buffer WoBuf: WO = 1,
///This is the Foo ref
ref FooRef = register Foo {
    const ADDRESS = 3;
    const RESET_VALUE = 0x000002;
}