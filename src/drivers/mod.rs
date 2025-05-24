pub mod uart;
pub mod virtio;
pub mod virtio_old;

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

pub struct MemoryRange {
    pub addr: usize,
    pub size_bytes: usize,
}

pub fn get_mem_addr(fdt: fdt::Fdt, compatible: &[&str]) -> Option<MemoryRange> {
    let node = fdt.find_compatible(compatible)?;
    // TODO: I dont know if I return all, will modify API when needed
    let memory_region = node.reg()?.next()?;

    let addr = memory_region.starting_address as usize;
    let size_bytes = memory_region.size?;

    Some(MemoryRange { addr, size_bytes })
}
