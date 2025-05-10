//! PCIe subsystem for the kleineOS kernel

use core::ptr::read_volatile;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::drivers::{DriverError, get_mem_addr};
use crate::vmem::{Mapper, Perms};

const COMPATIBLE: &[&str] = &["pci-host-ecam-generic"];

pub trait PciDeviceInit {
    // pair of (vendor_id, device_id) that this device is for
    fn id_pair(&self) -> (u16, u16);
    fn init(&self, header: PcieEcamHeader, ecam: PcieEcam);
}

/// This struct is used for passing the correct [DeviceStub] to the correct PcieDriver
pub struct PcieManager<'a> {
    devices: Vec<PcieEcamHeader>,
    drivers: BTreeMap<(u16, u16), &'a dyn PciDeviceInit>,
    ecam: PcieEcam,
}

impl<'a> PcieManager<'a> {
    pub fn new(ecam: PcieEcam) -> Self {
        let devices = enumerate(ecam);
        let drivers = BTreeMap::new();

        Self {
            devices,
            drivers,
            ecam,
        }
    }

    pub fn register_driver<T: PciDeviceInit>(&mut self, driver: &'a T) {
        let pair = driver.id_pair();
        self.drivers.insert(pair, driver);
    }

    pub fn init_drivers(self) {
        for device in self.devices.iter() {
            let id_pair = (device.vendor_id, device.device_id);
            if let Some(driver) = self.drivers.get(&id_pair) {
                driver.init(*device, self.ecam);
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PcieEcam {
    base_addr: usize,
}

impl PcieEcam {
    /// Read 32 bits via the PCIe ECAM interface
    pub fn read_word(self, bus: u8, device: u8, func: u8, offset: u8) -> u32 {
        let word_addr = self.base_addr
            + ((bus as usize) << 20)
            + ((device as usize) << 15)
            + ((func as usize) << 12)
            + ((offset as usize) & 0xFC); // align to 4-bit byte boundry
        unsafe { read_volatile(word_addr as *const u32) }
    }

    pub fn read_register(self, bus: u8, device: u8, register: u8) -> u32 {
        Self::read_word(self, bus, device, 0, register * 4)
    }

    pub fn get_common_dev_info(self, bus: u8, device: u8) -> Option<PcieEcamHeader> {
        // register 0x0: device_id ++ vendor_id
        let cache = self.read_register(bus, device, 0x0);
        let vendor_id = cache as u16;

        // device does not exist
        if vendor_id == 0xFFFF {
            return None;
        }

        let device_id = (cache >> 16) as u16;

        // register 0x1: status ++ command
        let cache = self.read_register(bus, device, 0x1);
        let status = (cache >> 16) as u16;
        let command = cache as u16;

        // register 0x2: class code ++ subclass ++ progif ++ revision
        let cache = self.read_register(bus, device, 0x2);
        let class_code = (cache >> 24) as u8;
        let subclass = (cache >> 16) as u8;
        let progif = (cache >> 8) as u8;
        let revision = cache as u8;

        // register 0x3: bist ++ header type ++ latency timer ++ cache line size
        let cache = self.read_register(bus, device, 0x3);
        let bist = (cache >> 24) as u8;
        let header_type = (cache >> 16) as u8;
        let latency_timer = (cache >> 8) as u8;
        let cache_line_size = cache as u8;

        let header = PcieEcamHeader {
            vendor_id,
            device_id,
            status,
            command,
            class_code,
            subclass,
            progif,
            revision,
            bist,
            header_type,
            latency_timer,
            cache_line_size,
        };

        Some(header)
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub struct PcieEcamHeader {
    pub vendor_id: u16,
    pub device_id: u16,
    pub status: u16,
    pub command: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub progif: u8,
    pub revision: u8,
    pub bist: u8,
    pub header_type: u8,
    pub latency_timer: u8,
    pub cache_line_size: u8,
}

pub fn init(fdt: fdt::Fdt, mapper: &mut Mapper) -> Result<PcieEcam, DriverError> {
    let mem_range = get_mem_addr(fdt, COMPATIBLE).ok_or(DriverError::DeviceNotFound)?;

    let base_addr = mem_range.addr;
    let pages = mem_range.size_bytes / crate::PAGE_SIZE;

    mapper.map(base_addr, base_addr, Perms::READ_WRITE, pages)?;

    let ecam = PcieEcam { base_addr };

    Ok(ecam)
}

/// enumerate over pcie devices using the bruteforce method
fn enumerate(ecam: PcieEcam) -> Vec<PcieEcamHeader> {
    let mut devices = alloc::vec![];

    for bus in 0..=255 {
        for device in 0..32 {
            if let Some(device) = ecam.get_common_dev_info(bus, device) {
                devices.push(device);
            }
        }
    }

    devices
}
