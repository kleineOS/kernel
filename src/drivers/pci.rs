use core::fmt::Display;

use alloc::vec::Vec;

use crate::vmem::{Mapper, Perms};

use super::DriverError;

const COMPATIBLE: &[&str] = &["pci-host-ecam-generic"];

pub struct Pci {}

impl Pci {
    pub fn enumerate(fdt: fdt::Fdt, mapper: &mut Mapper) -> Result<Vec<Device>, DriverError> {
        let base_addr = get_mem_addr(fdt).ok_or(DriverError::DeviceNotFound)?;
        mapper.map(base_addr, base_addr, Perms::READ_WRITE, 1)?;

        let devices = Self::bruteforce(base_addr);

        Ok(devices)
    }

    fn bruteforce(ecam_base: usize) -> Vec<Device> {
        let mut devices = alloc::vec![];

        for bus in 0..=255 {
            for device in 0..32 {
                if let Some(device) = Self::check_device(ecam_base, bus, device) {
                    devices.push(device);
                }
            }
        }

        devices
    }

    fn check_device(ecam_base: usize, bus: u8, device: u8) -> Option<Device> {
        let vendor_id = Self::read_word(ecam_base, bus, device, 0, 0x00);

        if vendor_id == 0xFFFF {
            return None; // No device
        }

        let device_id = Self::read_word(ecam_base, bus, device, 0, 0x02);
        let class_code = Self::read_word(ecam_base, bus, device, 0, 0x0A); // upper byte is class, lower is subclass
        let header_type = Self::read_word(ecam_base, bus, device, 0, 0x0E);

        Some(Device {
            device,
            vendor_id,
            device_id,
            class_code,
            header_type,
        })
    }

    fn read_word(ecam_base: usize, bus: u8, device: u8, func: u8, offset: u8) -> u16 {
        let address = ecam_base
            + ((bus as usize) << 20)
            + ((device as usize) << 15)
            + ((func as usize) << 12)
            + ((offset as usize) & 0xFFC); // align to 4-byte boundary

        let value = unsafe { core::ptr::read_volatile(address as *const u32) };

        if (offset & 2) == 0 {
            (value & 0xFFFF) as u16
        } else {
            ((value >> 16) & 0xFFFF) as u16
        }
    }
}

fn get_mem_addr(fdt: fdt::Fdt) -> Option<usize> {
    let node = fdt.find_compatible(COMPATIBLE)?;
    let memory_region = node.reg().into_iter().flatten().next()?;

    let address = memory_region.starting_address as usize;

    Some(address)
}

#[derive(Debug, Clone, Copy)]
pub struct Device {
    pub device: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u16,
    pub header_type: u16,
}

impl Display for Device {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "bus=0, device={}, vendor=0x{:04x}, device=0x{:04x}, class=0x{:02x}, subclass=0x{:02x}, header_type=0x{:04x}",
            self.device,
            self.vendor_id,
            self.device_id,
            self.class_code >> 8,
            self.class_code & 0xFF,
            self.header_type
        )
    }
}
