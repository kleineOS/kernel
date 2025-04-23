//! A driver for the 16550A UART device

use crate::vmem::{MemMapReq, Perms};
use errors::*;

const COMPATIBLE: &[&str] = &["ns16550a"];

pub fn init<F: FnMut(MemMapReq)>(fdt: fdt::Fdt, mut dmap: F) -> Result<(), ErrorKind> {
    let base_addr = get_mem_addr(fdt).ok_or(ErrorKind::DeviceNotFound)?;

    let map = MemMapReq {
        paddr: base_addr,
        vaddr: base_addr,
        pages: 1,
        perms: Perms::READ_WRITE,
    };

    Ok(())
}

fn get_mem_addr(fdt: fdt::Fdt) -> Option<usize> {
    let node = fdt.find_compatible(COMPATIBLE)?;
    let memory_region = node.reg().into_iter().flatten().next()?;

    assert_eq!(memory_region.size, Some(256));

    let address = memory_region.starting_address as usize;

    Some(address)
}

mod errors {

    #[derive(Debug)]
    pub enum ErrorKind {
        DeviceNotFound,
    }

    impl core::fmt::Display for ErrorKind {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            use ErrorKind::*;

            match self {
                DeviceNotFound => write!(f, "Device not found"),
            }
        }
    }

    impl core::error::Error for ErrorKind {}
}
