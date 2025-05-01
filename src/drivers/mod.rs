pub mod uart;

#[derive(Debug, thiserror::Error)]
pub enum DriverError {
    #[error("Device not found")]
    DeviceNotFound,
    #[error("Driver has not been initialised")]
    DriverUninitialised,
    #[error("Driver has already been initialised")]
    AlreadyInitialised,
}
