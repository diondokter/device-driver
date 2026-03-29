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
        _size_bits: u32,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        self.device_memory[address as usize..][..data.len()].copy_from_slice(data);

        Ok(())
    }

    fn read_register(
        &mut self,
        address: Self::AddressType,
        _size_bits: u32,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
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
            byte-order: BE,
            register-address-type: u8,

            /// This is the Foo register
            register Foo {
                address: 0,
                fields: fieldset FooFields {
                    size-bits: 24,
                    field value 23:0 -> uint,
                }
            },
            /// This is the Foo register
            register FooRepeated[4*3] {
                address: 3,
                fields: FooFields,
            },
            fieldset BarFields {
                size-bits: 16,
                field value 15:0 -> uint,
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

        unimplemented!("Non-multiple of 8 bit fieldsets cannot (yet) be packed");
    }

    fn unpack_r(&mut self) {
        if Self::GAP_BITS == 0 {
            return;
        }

        unimplemented!("Non-multiple of 8 bit fieldsets cannot (yet) be packed");
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
                core::mem::size_of::<Self>(),
            )
        }
    }

    fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                (&raw mut *self).cast::<u8>(),
                core::mem::size_of::<Self>(),
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
    let mut foo1_in = FooFields::new();
    foo1_in.set_value(0x123456);
    let mut foo2_in = FooFields::new();
    foo2_in.set_value(0x123456);
    let mut bar_in = BarFields::new();
    bar_in.set_value(0x1234);

    let mfs = MultiFS::from((foo1_in, foo2_in));
    let mfs = MultiFS::from((mfs, bar_in));

    assert_eq!(size_bits_of_val(&mfs), 24 + 24 + 16);
    assert_eq!(core::mem::size_of_val(&mfs), 3 + 3 + 2);

    assert_eq!(
        mfs.get_inner_buffer(),
        &[0x12, 0x34, 0x56, 0x12, 0x34, 0x56, 0x12, 0x34]
    );

    let (foo1, foo2, bar) = mfs.to_tuple();

    assert_eq!(foo1, foo1_in);
    assert_eq!(foo2, foo2_in);
    assert_eq!(bar, bar_in);
}

fn size_bits_of_val<T: FieldSet>(_: &T) -> u32 {
    T::SIZE_BITS
}
