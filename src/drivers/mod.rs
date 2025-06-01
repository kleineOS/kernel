use alloc::collections::{BTreeMap, BTreeSet};

use crate::systems::pci::{Device, PciMemory};

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

pub fn allocate_bar_addrs(
    bars: BTreeSet<u8>,
    device: &Device,
    mem: &mut PciMemory,
) -> Result<BTreeMap<u8, usize>, DriverError> {
    // to quote osdev.wiki:
    // > Before attempting to read the information about the BAR, make sure to disable both I/O and
    // > memory decode in the command byte. /* ... */ This is needed as some devices are known to
    // > decode the write of all ones to the register as an (unintended) access.
    device.disable_io_space();
    device.disable_mem_space();

    let mut bar_addrs = BTreeMap::<u8, usize>::new();
    for bar_nr in bars {
        let (is_64_bits, size) = device.get_bar_size(bar_nr);

        let address = mem.allocate(size as usize, is_64_bits);
        let address = address.ok_or(DriverError::OutOfMemoryPci)?;

        bar_addrs.insert(bar_nr, address);

        if is_64_bits {
            let addr_hi = (address >> 32) as u32;
            let addr_lo = address as u32;

            assert_eq!(address, ((addr_hi as usize) << 32) | (addr_lo as usize));

            device.write_bar(bar_nr + 1, addr_hi);
            device.write_bar(bar_nr, addr_lo);
        } else {
            log::warn!("[VIRTIO] 32-bit memory address are not tested, but the device requests it");
            device.write_bar(bar_nr, address as u32);
        }
    }

    device.enable_mem_space();

    Ok(bar_addrs)
}

/// Java, C#, Kotlin, Python: "Look at what they need, to mimic a fraction of our power"
pub mod regcell {
    use core::marker::PhantomData;

    pub struct RO;
    pub struct RW;

    /// # usage
    /// ```
    /// struct SampleRegStruct {
    ///     // you can use .get and .set on this register
    ///     device_feature_select: RegCell<u32, RW>,
    ///     // you can only use .get this register
    ///     device_feature: RegCell<u32>,
    /// }
    /// ```
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
