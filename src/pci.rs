//! PCIe subsystem for the kleineOS kernel
//! current version: 0.1-dev

use core::ptr::read_volatile;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::drivers::{DriverError, get_mem_addr};
use crate::vmem::{Mapper, Perms};

const COMPATIBLE: &[&str] = &["pci-host-ecam-generic"];

pub trait PciDeviceInit {
    /// pair of (vendor_id, device_id) that this device is for
    fn id_pair(&self) -> (u16, u16);
    fn init(&self, header: PcieEcamHeader, ecam: PcieEcam, fdt: fdt::Fdt);
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

    pub fn init_drivers(self, fdt: fdt::Fdt) {
        for device in self.devices.iter() {
            let id_pair = (device.vendor_id, device.device_id);
            if let Some(driver) = self.drivers.get(&id_pair) {
                driver.init(*device, self.ecam, fdt);
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PcieEcam {
    base_addr: usize,
}

// macro_rules! pcie_register {
//     ($x:ident, $offset:expr, $type:ty) => {
//         pub fn $x(self, bus: u8, device: u8) -> $type {
//             self.read_word(bus, device, 0, $offset) as $type
//         }
//     };
// }

impl PcieEcam {
    pub fn address(self, bus: u8, device: u8, func: u8) -> usize {
        self.base_addr
            + ((bus as usize) << 20)
            + ((device as usize) << 15)
            + ((func as usize) << 12)
    }

    /// Read 32 bits via the PCIe ECAM interface
    pub fn read_word(self, bus: u8, device: u8, func: u8, offset: u8) -> u32 {
        // add offset and align to 4-bit byte boundry
        let word_addr = self.address(bus, device, func) + ((offset as usize) & 0xFC);
        unsafe { read_volatile(word_addr as *const u32) }
    }

    // pcie_register!(read_device_id, 0x0, u16);
    // pcie_register!(read_vendor_id, 0x2, u16);

    pub fn read_register(self, bus: u8, device: u8, register: u8) -> u32 {
        Self::read_word(self, bus, device, 0, register * 4)
    }

    /// get device info that is common accross most PCIe devices
    /// going to be merged with get_more_dev_info in the next version of this subsystem
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

            bus_nr: bus,
            device_nr: device,
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
    // --- idk if this should be here ---
    pub bus_nr: u8,
    pub device_nr: u8,
}

#[derive(Debug)]
#[allow(unused)]
pub struct GeneralDevInfo {
    pub base_addrs: [u32; 6],
    pub subsystem_id: u16,
    pub cardbus_cis_ptr: u32,
    pub subsystem_vendor_id: u16,
    pub exp_rom_base_addr: u32,
    pub capabilities_ptr: u8,
    pub max_latency: u8,
    pub min_grant: u8,
    pub interrupt_pin: u8,
    pub interrupt_line: u8,
}

#[derive(Debug)]
pub enum MoreDevInfo {
    GeneralDev(GeneralDevInfo),
    Pci2PciBridge,
    Pci2CardBusBridge,
}

impl PcieEcamHeader {
    pub fn status_capabilities_list(self) -> bool {
        let mask = 0b1000000000010000;
        let bit = (self.status & mask) >> 4;
        assert!(bit <= 1, "we should have extracted a single bit {bit}");
        bit == 1
    }

    fn read_general_dev_info(self, ecam: PcieEcam) -> GeneralDevInfo {
        let bus = self.bus_nr;
        let device = self.device_nr;

        let mut base_addrs = [0; 6];
        base_addrs.iter_mut().enumerate().for_each(|(i, value)| {
            *value = ecam.read_register(bus, device, 0x4 + (i as u8));
        });

        let cardbus_cis_ptr = ecam.read_register(bus, device, 0xa);

        let cache = ecam.read_register(bus, device, 0xb);
        let subsystem_id = (cache >> 16) as u16;
        let subsystem_vendor_id = cache as u16;

        let exp_rom_base_addr = ecam.read_register(bus, device, 0xc);

        let capabilities_ptr = ecam.read_register(bus, device, 0xd) as u8;

        let cache = ecam.read_register(bus, device, 0xf);
        let max_latency = (cache >> 24) as u8;
        let min_grant = (cache >> 16) as u8;
        let interrupt_pin = (cache >> 8) as u8;
        let interrupt_line = cache as u8;

        GeneralDevInfo {
            base_addrs,
            cardbus_cis_ptr,
            subsystem_id,
            subsystem_vendor_id,
            exp_rom_base_addr,
            capabilities_ptr,
            max_latency,
            min_grant,
            interrupt_pin,
            interrupt_line,
        }
    }

    /// get device info based on the header type
    /// going to be merged with get_common_dev_info in the next version of this subsystem
    pub fn get_more_dev_info(self, ecam: PcieEcam) -> MoreDevInfo {
        match self.header_type {
            0x0 => MoreDevInfo::GeneralDev(self.read_general_dev_info(ecam)),
            0x1 => MoreDevInfo::Pci2PciBridge,
            0x2 => MoreDevInfo::Pci2CardBusBridge,
            _ => unreachable!(),
        }
    }
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

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub enum VirtioPciCapCfg {
    /// Common configuration
    Common = 1,
    /// Notifications
    Notify = 2,
    /// ISR Status
    Isr = 3,
    /// Device specific configuration
    Device = 4,
    /// PCI configuration access
    Pci = 5,
    /// Shared memory region
    SharedMem = 8,
    /// Vendor-specific data
    Vendor = 9,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub struct VirtioPciCap {
    pub cap_vndr: u8,
    pub cap_next: u8,
    pub cap_len: u8,
    pub cfg_type: VirtioPciCapCfg,
    pub bar: u8,
    pub id: u8,
    _padding: [u8; 2],
    pub offset_le: u32,
    pub length_le: u32,
}

pub fn enumerate_capabilities(base: usize, offset: usize) -> [Option<VirtioPciCap>; 10] {
    let mut caps = [None; 10];
    let mut i = 0;

    let mut cap_base = base + offset;
    loop {
        let cap_id = unsafe { read_volatile(cap_base as *const u8) };

        if cap_id == 0x09 {
            let capabilities = unsafe { read_volatile(cap_base as *const VirtioPciCap) };
            //log::trace!("{cap_base:#x}: {capabilities:#x?}");
            caps[i] = Some(capabilities);
            i += 1;
        }

        let next_cap = unsafe { read_volatile((cap_base + 1) as *const u8) };

        if next_cap == 0 {
            break;
        }

        cap_base = base + next_cap as usize;
    }

    caps
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn todo() {
        todo!("write test cases for PCIe Manager");
    }
}
