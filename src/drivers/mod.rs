pub mod pci;
pub mod uart;
pub mod virtio;

#[derive(Debug, thiserror::Error)]
#[allow(unused)]
pub enum DriverError {
    #[error("Device not found")]
    DeviceNotFound,
    #[error("Driver has not been initialised")]
    DriverUninitialised,
    #[error("Driver has already been initialised")]
    AlreadyInitialised,
    #[error("Unimplimented functionality")]
    Unimplimented,
    #[error("Could not map device address due to error: {error}")]
    MapError {
        #[from]
        error: crate::vmem::MapError,
    },
}
