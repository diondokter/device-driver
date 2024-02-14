///Doc comment for the ID register
pub fn id(
    &mut self,
) -> device_driver::RegisterOperation<'_, Self, Id, { Id::SIZE_BYTES }> {
    device_driver::RegisterOperation::new(self)
}
///Baudrate register
pub fn baudrate(
    &mut self,
) -> device_driver::RegisterOperation<'_, Self, Baudrate, { Baudrate::SIZE_BYTES }> {
    device_driver::RegisterOperation::new(self)
}
