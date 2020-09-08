use core::fmt::Debug;

#[derive(Debug)]
pub enum RegisterError<IE> {
    InvalidValue,
    HardwareError(IE),
}

impl<IE> From<IE> for RegisterError<IE> {
    fn from(value: IE) -> Self {
        RegisterError::HardwareError(value)
    }
}

pub trait RegisterInterface {
    type Address;
    type InterfaceError: Debug;

    fn read_register(
        &mut self,
        address: Self::Address,
        value: &mut [u8],
    ) -> Result<(), Self::InterfaceError>;

    fn write_register(
        &mut self,
        address: Self::Address,
        value: &[u8],
    ) -> Result<(), Self::InterfaceError>;
}

#[macro_export]
macro_rules! implement_registers {
    (
        $device_name:ident.$registers_name:ident<$register_address_type:ty> = {
            $(
                $register_name:ident($register_access_specifier:tt, $register_address:expr, $register_size:expr) = {

                }
            ),*
        }
    ) => {
        pub mod $registers_name {
            use device_driver::ll::register::{RegisterInterface};
            use super::*;
            use device_driver::hl::Device;
            use device_driver::implement_reg_accessor;

            impl<'a, I> $device_name<I>
            where
                I: 'a + RegisterInterface<Address = $register_address_type>,
            {
                pub fn $registers_name(&'a mut self) -> Registers<'a, I> {
                    Registers::new(&mut self.interface)
                }
            }

            pub struct RegAccessor<'a, I, R, W>
            where
                I: 'a + RegisterInterface<Address = $register_address_type>,
            {
                interface: &'a mut I,
                phantom: core::marker::PhantomData<(R, W)>,
            }

            impl<'a, I, R, W> RegAccessor<'a, I, R, W>
            where
                I: 'a + RegisterInterface<Address = $register_address_type>,
            {
                fn new(interface: &'a mut I) -> Self {
                    Self {
                        interface,
                        phantom: Default::default(),
                    }
                }
            }

            pub struct Registers<'a, I>
            where
                I: 'a + RegisterInterface<Address = $register_address_type>,
            {
                interface: &'a mut I,
            }

            impl<'a, I> Registers<'a, I>
            where
                I: 'a + RegisterInterface<Address = $register_address_type>,
            {
                fn new(interface: &'a mut I) -> Self {
                    Self { interface }
                }

                $(
                    pub fn $register_name(&'a mut self) -> RegAccessor<'a, I, $register_name::R, $register_name::W> {
                        RegAccessor::new(&mut self.interface)
                    }
                )*
            }

            $(
                pub mod $register_name {
                    use super::*;

                    pub struct R([u8; $register_size]);
                    pub struct W([u8; $register_size]);

                    impl<'a, I> RegAccessor<'a, I, R, W>
                    where
                        I: RegisterInterface<Address = $register_address_type>,
                    {
                        implement_reg_accessor!($register_access_specifier, $register_address);
                    }

                    impl R {
                        fn zero() -> Self {
                            Self([0; $register_size])
                        }
                    }
                    impl W {
                        fn zero() -> Self {
                            Self([0; $register_size])
                        }
                    }
                }
            )*
        }
    };
}

#[macro_export]
macro_rules! implement_reg_accessor {
    (RO, $address:expr) => {
        pub fn read(&mut self) -> Result<R, RegisterError<I::InterfaceError>> {
            let mut r = R::zero();
            self.interface.read_register($address, &mut r.0)?;
            Ok(r)
        }
    };
    (WO, $address:expr) => {
        pub fn write<F>(&mut self, f: F) -> Result<(), RegisterError<I::InterfaceError>>
        where
            F: FnOnce(W) -> W
        {
            let w = f(W::zero());
            self.interface.write_register($address, &w.0)?;
            Ok(())
        }
    };
    (RW, $address:expr) => {
        implement_reg_accessor!(RO, $address);
        implement_reg_accessor!(WO, $address);

        pub fn modify<F>(&mut self, f: F) -> Result<(), RegisterError<I::InterfaceError>>
        where
            F: FnOnce(R, W) -> W
        {
            let r = self.read()?;
            let w = W(r.0.clone());

            let w = f(r, w);

            self.write(|_| w)?;
            Ok(())
        }
    };
}
