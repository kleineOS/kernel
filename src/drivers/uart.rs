//! A driver for the 16550A UART device
//!
//! TODO: will continue the impl once I have some other stuff done

use super::{CharDriver, Driver, DriverError};
use crate::vmem::{Mapper, Perms};

const COMPATIBLE: &[&str] = &["ns16550a"];

pub struct UartDriver {
    base_addr: usize,
}

impl UartDriver {}

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
