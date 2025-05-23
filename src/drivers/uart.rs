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
        let mem_range = super::get_mem_addr(fdt, COMPATIBLE).ok_or(DriverError::DeviceNotFound)?;

        let base_addr = mem_range.addr;

        Self::init_direct(base_addr)?;

        assert_eq!(mem_range.size_bytes, 256);
        mapper.map(base_addr, base_addr, Perms::READ_WRITE, 1)?;

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
