//! A driver for the 16550A UART device
//!
//! TODO: will continue the impl once I have some other stuff done

use core::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use alloc::boxed::Box;

use super::DriverError;
use crate::vmem::{Mapper, Perms};

const COMPATIBLE: &[&str] = &["ns16550a"];

static DRIVER_PTR: AtomicPtr<CharDriver> = AtomicPtr::new(null_mut());

/// Character device driver for ns16550a compatible UART devices
#[derive(Debug)]
pub struct CharDriver {
    base_addr: usize,
}

impl CharDriver {
    pub fn init(fdt: fdt::Fdt, mapper: &mut Mapper) -> Result<(), DriverError> {
        let base_addr = get_mem_addr(fdt).ok_or(DriverError::DeviceNotFound)?;

        Self::init_direct(base_addr)?;

        mapper.map(base_addr, base_addr, Perms::READ_WRITE, 1);

        Ok(())
    }

    pub fn log_addr() -> Result<(), DriverError> {
        let this = Self::get_instance().ok_or(DriverError::DriverUninitialised)?;

        let addr = this.base_addr;
        log::info!("uart driver base_addr={addr:#x}",);

        Ok(())
    }

    fn get_instance() -> Option<&'static mut Self> {
        unsafe { DRIVER_PTR.load(Ordering::Relaxed).as_mut() }
    }

    fn init_direct(base_addr: usize) -> Result<(), DriverError> {
        let driver = Box::new(CharDriver { base_addr });
        let driver_ptr = Box::leak(driver);

        // only load value if previous value is null_mut
        let res = DRIVER_PTR.compare_exchange(
            null_mut(),
            driver_ptr,
            Ordering::AcqRel,
            Ordering::Relaxed,
        );

        if res.is_err() {
            // we need to make sure the drop logic is run
            let _ = unsafe { Box::from_raw(driver_ptr) };
            return Err(DriverError::AlreadyInitialised);
        }

        Ok(())
    }
}

fn get_mem_addr(fdt: fdt::Fdt) -> Option<usize> {
    let node = fdt.find_compatible(COMPATIBLE)?;
    let memory_region = node.reg().into_iter().flatten().next()?;

    assert_eq!(memory_region.size, Some(256));

    let address = memory_region.starting_address as usize;

    Some(address)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    pub fn double_init() {
        crate::println!("[drivers::uart::tests::double_init]");

        // the driver is initialised before test cases are called
        // let driver = CharDriver::init_direct(0x10000000);
        // assert!(driver.is_ok(), "is_ok failed {driver:?}");

        let driver = CharDriver::init_direct(0x10000000);
        assert!(driver.is_err(), "is_err failed {driver:?}");
    }
}
