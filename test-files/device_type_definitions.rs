///Doc comment for the ID register
pub struct Id {
    bits: device_driver::bitvec::array::BitArray<[u8; Self::SIZE_BYTES]>,
}
impl device_driver::Register<{ Self::SIZE_BYTES }> for Id {
    const ZERO: Self = Self {
        bits: device_driver::bitvec::array::BitArray::ZERO,
    };
    type AddressType = u8;
    const ADDRESS: Self::AddressType = 12;
    type RWType = device_driver::ReadOnly;
    const SIZE_BITS: usize = 24;
    type WriteFields = id::W;
    type ReadFields = id::R;
    fn bits_mut(
        &mut self,
    ) -> &mut device_driver::bitvec::array::BitArray<[u8; Self::SIZE_BYTES]> {
        &mut self.bits
    }
    fn bits(&self) -> &device_driver::bitvec::array::BitArray<[u8; Self::SIZE_BYTES]> {
        &self.bits
    }
    fn reset_value() -> Self
    where
        Self: Sized,
    {
        Self {
            bits: device_driver::bitvec::array::BitArray::new([0u8, 0u8, 5u8]),
        }
    }
}
impl Id {
    pub const SIZE_BYTES: usize = 3;
}
///Implementation of R and W types for [Id]
pub mod id {
    use super::*;
    ///Write struct for [Id]
    pub struct W {
        inner: Id,
    }
    impl From<Id> for W {
        fn from(val: Id) -> Self {
            Self { inner: val }
        }
    }
    impl From<W> for Id {
        fn from(val: W) -> Self {
            val.inner
        }
    }
    impl core::ops::Deref for W {
        type Target = Id;
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
    impl core::ops::DerefMut for W {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }
    impl W {
        pub const SIZE_BYTES: usize = 3;
        ///Doc comment for the manufacturer field
        pub fn manufacturer(&mut self, data: Manufacturer) -> &mut Self {
            device_driver::write_field::<
                Self,
                _,
                Manufacturer,
                u16,
                0,
                16,
                { Self::SIZE_BYTES },
            >(self, data)
        }
        pub fn version(&mut self, data: u8) -> &mut Self {
            device_driver::write_field_no_convert::<
                Self,
                _,
                u8,
                16,
                20,
                { Self::SIZE_BYTES },
            >(self, data)
        }
        pub fn edition(&mut self, data: Edition) -> &mut Self {
            device_driver::write_field::<
                Self,
                _,
                Edition,
                u8,
                20,
                24,
                { Self::SIZE_BYTES },
            >(self, data)
        }
        ///Doc comment for the manufacturer field
        pub fn get_manufacturer(
            &self,
        ) -> Result<Manufacturer, <Manufacturer as TryFrom<u16>>::Error> {
            device_driver::read_field::<
                Self,
                _,
                Manufacturer,
                u16,
                0,
                16,
                { Self::SIZE_BYTES },
            >(self)
        }
        pub fn get_version(&self) -> u8 {
            device_driver::read_field_no_convert::<
                Self,
                _,
                u8,
                16,
                20,
                { Self::SIZE_BYTES },
            >(self)
        }
        pub fn get_edition(&self) -> Result<Edition, <Edition as TryFrom<u8>>::Error> {
            device_driver::read_field::<
                Self,
                _,
                Edition,
                u8,
                20,
                24,
                { Self::SIZE_BYTES },
            >(self)
        }
    }
    ///Read struct for [Id]
    pub struct R {
        inner: Id,
    }
    impl From<Id> for R {
        fn from(val: Id) -> Self {
            Self { inner: val }
        }
    }
    impl From<R> for Id {
        fn from(val: R) -> Self {
            val.inner
        }
    }
    impl core::fmt::Debug for R {
        fn fmt(
            &self,
            fmt: &mut core::fmt::Formatter<'_>,
        ) -> Result<(), core::fmt::Error> {
            fmt.debug_struct("Id")
                .field("manufacturer", &self.manufacturer())
                .field("version", &self.version())
                .field("edition", &self.edition())
                .finish()
        }
    }
    impl core::ops::Deref for R {
        type Target = Id;
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
    impl core::ops::DerefMut for R {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }
    impl R {
        pub const SIZE_BYTES: usize = 3;
        ///Doc comment for the manufacturer field
        pub fn manufacturer(
            &self,
        ) -> Result<Manufacturer, <Manufacturer as TryFrom<u16>>::Error> {
            device_driver::read_field::<
                Self,
                _,
                Manufacturer,
                u16,
                0,
                16,
                { Self::SIZE_BYTES },
            >(self)
        }
        pub fn version(&self) -> u8 {
            device_driver::read_field_no_convert::<
                Self,
                _,
                u8,
                16,
                20,
                { Self::SIZE_BYTES },
            >(self)
        }
        pub fn edition(&self) -> Result<Edition, <Edition as TryFrom<u8>>::Error> {
            device_driver::read_field::<
                Self,
                _,
                Edition,
                u8,
                20,
                24,
                { Self::SIZE_BYTES },
            >(self)
        }
    }
}
#[derive(
    device_driver::num_enum::TryFromPrimitive,
    device_driver::num_enum::IntoPrimitive,
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq
)]
#[repr(u8)]
pub enum Edition {
    One = 1,
    Two,
    ///Test!
    Five = 5,
    #[num_enum(default)]
    Others,
}
///Baudrate register
pub struct Baudrate {
    bits: device_driver::bitvec::array::BitArray<[u8; Self::SIZE_BYTES]>,
}
impl device_driver::Register<{ Self::SIZE_BYTES }> for Baudrate {
    const ZERO: Self = Self {
        bits: device_driver::bitvec::array::BitArray::ZERO,
    };
    type AddressType = u8;
    const ADDRESS: Self::AddressType = 42;
    type RWType = device_driver::ReadWrite;
    const SIZE_BITS: usize = 16;
    type WriteFields = baudrate::W;
    type ReadFields = baudrate::R;
    fn bits_mut(
        &mut self,
    ) -> &mut device_driver::bitvec::array::BitArray<[u8; Self::SIZE_BYTES]> {
        &mut self.bits
    }
    fn bits(&self) -> &device_driver::bitvec::array::BitArray<[u8; Self::SIZE_BYTES]> {
        &self.bits
    }
}
impl Baudrate {
    pub const SIZE_BYTES: usize = 2;
}
///Implementation of R and W types for [Baudrate]
pub mod baudrate {
    use super::*;
    ///Write struct for [Baudrate]
    pub struct W {
        inner: Baudrate,
    }
    impl From<Baudrate> for W {
        fn from(val: Baudrate) -> Self {
            Self { inner: val }
        }
    }
    impl From<W> for Baudrate {
        fn from(val: W) -> Self {
            val.inner
        }
    }
    impl core::ops::Deref for W {
        type Target = Baudrate;
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
    impl core::ops::DerefMut for W {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }
    impl W {
        pub const SIZE_BYTES: usize = 2;
        ///Baudrate value
        pub fn value(&mut self, data: u16) -> &mut Self {
            device_driver::write_field_no_convert::<
                Self,
                _,
                u16,
                0,
                16,
                { Self::SIZE_BYTES },
            >(self, data)
        }
        ///Baudrate value
        pub fn get_value(&self) -> u16 {
            device_driver::read_field_no_convert::<
                Self,
                _,
                u16,
                0,
                16,
                { Self::SIZE_BYTES },
            >(self)
        }
    }
    ///Read struct for [Baudrate]
    pub struct R {
        inner: Baudrate,
    }
    impl From<Baudrate> for R {
        fn from(val: Baudrate) -> Self {
            Self { inner: val }
        }
    }
    impl From<R> for Baudrate {
        fn from(val: R) -> Self {
            val.inner
        }
    }
    impl core::fmt::Debug for R {
        fn fmt(
            &self,
            fmt: &mut core::fmt::Formatter<'_>,
        ) -> Result<(), core::fmt::Error> {
            fmt.debug_struct("Baudrate").field("value", &self.value()).finish()
        }
    }
    impl core::ops::Deref for R {
        type Target = Baudrate;
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
    impl core::ops::DerefMut for R {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }
    impl R {
        pub const SIZE_BYTES: usize = 2;
        ///Baudrate value
        pub fn value(&self) -> u16 {
            device_driver::read_field_no_convert::<
                Self,
                _,
                u16,
                0,
                16,
                { Self::SIZE_BYTES },
            >(self)
        }
    }
}
