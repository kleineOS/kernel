//! A driver for the 16550A UART device
//!
//! TODO: will continue the impl once I have some other stuff done

use core::{ptr::null_mut, sync::atomic::AtomicPtr};

use super::{CharDriver, Driver, DriverError};
use crate::vmem::{Mapper, Perms};

const COMPATIBLE: &[&str] = &["ns16550a"];

static DRIVER_PTR: AtomicPtr<UartDriver> = AtomicPtr::new(null_mut());

pub struct UartDriver {
    base_addr: usize,
}

impl UartDriver {
    #[cfg(test)]
    fn init_test(balloc: &mut crate::allocator::BitMapAlloc, base_addr: usize) {
        use core::sync::atomic::Ordering;

        let struct_addr: usize = balloc.alloc(1);

        let driver = UartDriver { base_addr };

        unsafe { core::ptr::write(struct_addr as *mut UartDriver, driver) }

        DRIVER_PTR.store(struct_addr as *mut UartDriver, Ordering::Relaxed);
    }
}

impl Driver for UartDriver {
    fn init(fdt: fdt::Fdt, mapper: &mut Mapper) -> Result<Self, DriverError> {
        let base_addr = get_mem_addr(fdt).ok_or(DriverError::DeviceNotFound)?;
        mapper.map(base_addr, base_addr, Perms::READ_WRITE, 1);

        Ok(Self { base_addr })
    }

    fn inithart() {}
}

impl CharDriver for UartDriver {
    fn put_char(&self, c: char) {
        todo!()
    }
}

fn get_mem_addr(fdt: fdt::Fdt) -> Option<usize> {
    let node = fdt.find_compatible(COMPATIBLE)?;
    let memory_region = node.reg().into_iter().flatten().next()?;

    assert_eq!(memory_region.size, Some(256));

    let address = memory_region.starting_address as usize;

    Some(address)
}

// #[cfg(test)]
// mod tests {
//     use core::sync::atomic::Ordering;
//
//     use crate::allocator::BitMapAlloc;
//
//     use super::*;
//
//     #[test_case]
//     fn hello() {
//         let mut balloc = unsafe { BitMapAlloc::init(crate::HEAP1_TOP) };
//         let mut b = balloc.lock();
//
//         // TODO: essentially, I should create a global_alloc impl, so I wont need to pass in balloc
//         // everywhere, especially in places like the drivers which can't be held down by some
//         // strict control flow of the balloc value
//
//         UartDriver::init_test(&mut b, 0xdeadbead);
//         let driver = super::DRIVER_PTR.load(Ordering::Relaxed);
//         unsafe { assert_eq!((*driver).base_addr, 0xdeadbead) };
//     }
// }
