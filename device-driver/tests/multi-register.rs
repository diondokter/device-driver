use device_driver::{FieldSet, RegisterInterface};

pub struct DeviceInterface {
    device_memory: [u8; 128],
}

impl Default for DeviceInterface {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceInterface {
    pub const fn new() -> Self {
        Self {
            device_memory: [0; 128],
        }
    }
}

impl RegisterInterface for DeviceInterface {
    type Error = ();
    type AddressType = u8;

    fn write_register(
        &mut self,
        address: Self::AddressType,
        size_bits: u32,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        assert_eq!(size_bits, 24);
        self.device_memory[address as usize..][..data.len()].copy_from_slice(data);

        Ok(())
    }

    fn read_register(
        &mut self,
        address: Self::AddressType,
        size_bits: u32,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
        assert_eq!(size_bits, 24);
        data.copy_from_slice(&self.device_memory[address as usize..][..data.len()]);
        Ok(())
    }
}

device_driver::compile!(
    options: [
        "defmt-feature=defmt"
    ],
    ddsl: "
        device MyTestDevice {
            byte-order: LE,
            register-address-type: u8,

            /// This is the Foo register
            register Foo {
                address: 0,
                fields: fieldset FooFields {
                    size-bits: 24,

                    /// This is a bool!
                    field value0 0 -> bool,
                    field value1 15:1 -> uint,
                    field value2 23:16 -> int,
                }
            },
            /// This is the Foo register
            register FooRepeated[4*3] {
                address: 3,
                fields: FooFields,
            },
            fieldset BarFields {
                size-bits: 8
            }
        }
    "
);

#[repr(C)]
struct MultiFS<L: FieldSet, R>(L, R);

impl<L: FieldSet, R: SimpleFieldSet> From<(L, R)> for MultiFS<L, R> {
    fn from(value: (L, R)) -> Self {
        let mut multi = MultiFS(value.0, value.1);
        multi.pack_r();
        multi
    }
}

impl<L: FieldSet, R: SimpleFieldSet> MultiFS<L, R> {
    const GAP_BITS: u32 = { core::mem::size_of::<L>() as u32 * 8 - L::SIZE_BITS };

    fn pack_r(&mut self) {
        if Self::GAP_BITS == 0 {
            return;
        }

        todo!("Shift the bits of R into the gap of L");
    }

    fn unpack_r(&mut self) {
        if Self::GAP_BITS == 0 {
            return;
        }

        todo!("Shift the bits of R from the gap of L to the normal position");
    }
}

impl<L: FieldSet, R: SimpleFieldSet> Default for MultiFS<L, R> {
    fn default() -> Self {
        let mut val = Self(Default::default(), Default::default());
        val.pack_r();
        val
    }
}

impl<L: FieldSet, R: SimpleFieldSet> FieldSet for MultiFS<L, R> {
    const SIZE_BITS: u32 = L::SIZE_BITS + R::SIZE_BITS;

    fn get_inner_buffer(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                (&raw const *self).cast::<u8>(),
                Self::SIZE_BITS.div_ceil(8) as usize,
            )
        }
    }

    fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                (&raw mut *self).cast::<u8>(),
                Self::SIZE_BITS.div_ceil(8) as usize,
            )
        }
    }
}

trait ToTuple {
    type Tuple;

    fn to_tuple(self) -> Self::Tuple;
}

impl<A: SimpleFieldSet> ToTuple for A {
    type Tuple = A;

    fn to_tuple(self) -> Self::Tuple {
        self
    }
}

impl<A: SimpleFieldSet, B: SimpleFieldSet> ToTuple for MultiFS<A, B> {
    type Tuple = (A, B);

    fn to_tuple(mut self) -> Self::Tuple {
        self.unpack_r();
        (self.0, self.1)
    }
}

impl<A: SimpleFieldSet, B: SimpleFieldSet, C: SimpleFieldSet> ToTuple
    for MultiFS<MultiFS<A, B>, C>
{
    type Tuple = (A, B, C);

    fn to_tuple(mut self) -> Self::Tuple {
        self.unpack_r();
        self.0.unpack_r();
        (self.0.0, self.0.1, self.1)
    }
}

impl<A: SimpleFieldSet, B: SimpleFieldSet, C: SimpleFieldSet, D: SimpleFieldSet> ToTuple
    for MultiFS<MultiFS<MultiFS<A, B>, C>, D>
{
    type Tuple = (A, B, C, D);

    fn to_tuple(mut self) -> Self::Tuple {
        self.unpack_r();
        self.0.unpack_r();
        self.0.0.unpack_r();
        (self.0.0.0, self.0.0.1, self.0.1, self.1)
    }
}

impl<A: SimpleFieldSet, B: SimpleFieldSet, C: SimpleFieldSet, D: SimpleFieldSet, E: SimpleFieldSet>
    ToTuple for MultiFS<MultiFS<MultiFS<MultiFS<A, B>, C>, D>, E>
{
    type Tuple = (A, B, C, D, E);

    fn to_tuple(mut self) -> Self::Tuple {
        self.unpack_r();
        self.0.unpack_r();
        self.0.0.unpack_r();
        self.0.0.0.unpack_r();
        (self.0.0.0.0, self.0.0.0.1, self.0.0.1, self.0.1, self.1)
    }
}

trait SimpleFieldSet: FieldSet {}

impl SimpleFieldSet for BarFields {}
impl SimpleFieldSet for FooFields {}

#[test]
fn simple_layout_ok() {
    let mfs = MultiFS(
        FooFields::from([0x00, 0x11, 0x22]),
        FooFields::from([0x33, 0x44, 0x55]),
    );
    let mfs = MultiFS::from((mfs, BarFields::from([0x66])));

    assert_eq!(
        mfs.get_inner_buffer(),
        &[0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66]
    );

    let (foo1, foo2, bar) = mfs.to_tuple();

    assert_eq!(foo1.bits, [0x00, 0x11, 0x22]);
    assert_eq!(foo2.bits, [0x33, 0x44, 0x55]);
    assert_eq!(bar.bits, [0x66]);
}
