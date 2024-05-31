use core::marker::PhantomData;

use embedded_io::ErrorKind;

use crate::{ReadCapability, WriteCapability};

pub trait BufferDevice {
    type Id: Copy;

    fn write(&mut self, id: Self::Id, buf: &[u8]) -> Result<usize, ErrorKind>;
    fn read(&mut self, id: Self::Id, buf: &mut [u8]) -> Result<usize, ErrorKind>;
}

pub trait AsyncBufferDevice {
    type Id: Copy;

    async fn write(&mut self, id: Self::Id, buf: &[u8]) -> Result<usize, ErrorKind>;
    async fn read(&mut self, id: Self::Id, buf: &mut [u8]) -> Result<usize, ErrorKind>;
}

pub struct BufferOperation<'a, D, Id, RWType> {
    device: &'a mut D,
    id: Id,
    _phantom: PhantomData<RWType>,
}

impl<'a, D, Id, RWType> BufferOperation<'a, D, Id, RWType> {
    #[doc(hidden)]
    pub fn new(device: &'a mut D, id: Id) -> Self {
        Self {
            device,
            id,
            _phantom: PhantomData,
        }
    }
}

impl<'a, D, Id, RWType> embedded_io::ErrorType for BufferOperation<'a, D, Id, RWType> {
    type Error = ErrorKind;
}

impl<'a, D, Id: Copy, RWType> embedded_io::Read for BufferOperation<'a, D, Id, RWType>
where
    D: BufferDevice<Id = Id>,
    RWType: ReadCapability,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.device.read(self.id, buf)
    }
}

impl<'a, D, Id: Copy, RWType> embedded_io_async::Read for BufferOperation<'a, D, Id, RWType>
where
    D: AsyncBufferDevice<Id = Id>,
    RWType: ReadCapability,
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.device.read(self.id, buf).await
    }
}

impl<'a, D, Id: Copy, RWType> embedded_io::Write for BufferOperation<'a, D, Id, RWType>
where
    D: BufferDevice<Id = Id>,
    RWType: WriteCapability,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.device.write(self.id, buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, D, Id: Copy, RWType> embedded_io_async::Write for BufferOperation<'a, D, Id, RWType>
where
    D: AsyncBufferDevice<Id = Id>,
    RWType: WriteCapability,
{
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.device.write(self.id, buf).await
    }
}
