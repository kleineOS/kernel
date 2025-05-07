//! PCIe subsystem for the kleineOS kernel

use core::ptr::read_volatile;

use crate::drivers::{DriverError, get_mem_addr};
use crate::vmem::{Mapper, Perms};

const COMPATIBLE: &[&str] = &["pci-host-ecam-generic"];

#[derive(Debug, Clone, Copy)]
struct PcieEcam {
    base_addr: usize,
}

impl PcieEcam {
    pub fn read_word(self, bus: usize, device: usize, function: usize) -> u32 {
        let word_addr = self.base_addr + (bus << 20 | device << 15 | function << 12);
        unsafe { read_volatile(word_addr as *const u32) }
    }
}

pub fn init(fdt: fdt::Fdt, mapper: &mut Mapper) -> Result<(), DriverError> {
    let mem_range = get_mem_addr(fdt, COMPATIBLE).ok_or(DriverError::DeviceNotFound)?;

    let base_addr = mem_range.addr;
    let pages = mem_range.size_bytes / crate::PAGE_SIZE;

    mapper
        .map(base_addr, base_addr, Perms::READ_WRITE, pages)
        .unwrap();

    let ecam = PcieEcam { base_addr };

    ecam.read_word(0, 0, 0);
    ecam.read_word(0, 1, 0);

    Ok(())
}
