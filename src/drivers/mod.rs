pub mod uart;
pub mod virtio;

#[derive(Debug, thiserror::Error)]
#[allow(unused)]
pub enum DriverError {
    #[error("Device not found")]
    DeviceNotFound,
    #[error("Device is invalid: {reason}")]
    InvalidDevice { reason: &'static str },
    #[error("Driver has not been initialised")]
    DriverUninitialised,
    #[error("Out of PCIe memory")]
    OutOfMemoryPci,
    #[error("Driver has already been initialised")]
    AlreadyInitialised,
    #[error("Unimplimented functionality")]
    Unimplimented,
    #[error("Could not map device address due to error: {error}")]
    MapError {
        #[from]
        error: crate::vmem::MapError,
    },
    #[error("Driver error: {0}")]
    OtherError(&'static str),
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

pub mod regcell {
    use core::marker::PhantomData;

    pub struct RO;
    pub struct RW;

    #[repr(C)]
    pub struct RegCell<T, C = RO> {
        inner: core::cell::UnsafeCell<T>,
        _constraint: PhantomData<C>,
    }

    impl<T, C> RegCell<T, C> {
        #[inline]
        pub fn get(&self) -> T {
            unsafe { core::ptr::read_volatile(self.inner.get()) }
        }
    }

    impl<T> RegCell<T, RW> {
        #[inline]
        pub fn set(&self, value: T) {
            unsafe { core::ptr::write_volatile(self.inner.get(), value) };
        }
    }

    impl<T: core::fmt::Debug + Copy, C> core::fmt::Debug for RegCell<T, C> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{:#x?}", &self.get())
        }
    }
}
