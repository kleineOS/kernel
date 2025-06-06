//! PCIe subsystem for the kleineOS kernel
//! current version: 0.2-dev
#![allow(unused)]

mod ecam;
mod pci_device;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

pub use self::{ecam::*, pci_device::*};
use crate::vmem::{Mapper, Perms};
use crate::{PAGE_SIZE, round_down_by};

const COMPATIBLE: &[&str] = &["pci-host-ecam-generic"];
const PCI_DEFAULT_MEM_SIZE: usize = PAGE_SIZE;

const RANGE_MMIO_64_BIT: u32 = 0b10;
const RANGE_MMIO_32_BIT: u32 = 0b11;
const RANGE_PIO: u32 = 0b01;
const RANGE_CONFIG: u32 = 0b00;

const OFFSET_VENDOR_ID: u8 = 0x0;
const OFFSET_DEVICE_ID: u8 = 0x2;
const OFFSET_COMMAND: u8 = 0x4;
// all six of these have a difference of 4 (bytes), as each field is 32-bits
const OFFSET_BARS: [u8; 6] = [0x10, 0x14, 0x18, 0x1C, 0x20, 0x24];

#[derive(Debug)]
pub struct PciSubsystem {
    mem: PciMemory,
    ecam: Ecam,
    devices: BTreeMap<(u16, u16), Device>,
}

impl PciSubsystem {
    pub fn init(fdt: fdt::Fdt, mapper: &mut Mapper) -> Option<Self> {
        let mem = PciMemory::parse_from_fdt(fdt)?;
        mem.map_memory(mapper);

        let ecam = Ecam::init(mem.base_address);

        let devices = enumerate_devices(ecam)
            .into_iter()
            .map(|device| {
                let vendor_id = device.vendor_id();
                let device_id = device.device_id();

                log::info!("[PCI] DEVICE FOUND: {vendor_id:04x}:{device_id:04x}");

                ((vendor_id, device_id), device)
            })
            .collect();

        log::info!("[PCI] PCI subsystem has been initialised");
        Some(Self { mem, ecam, devices })
    }

    pub fn init_driver<F: FnOnce(Device, &mut PciMemory)>(&mut self, id: (u16, u16), init_fn: F) {
        if let Some(device) = self.devices.remove(&id) {
            init_fn(device, &mut self.mem);
        }
    }
}

/// Handles allocation of physical memory for PCI(e)
#[derive(Debug)]
#[allow(unused)]
pub struct PciMemory {
    base_address: usize,
    base_address_size: usize,

    mmio_32_bit: Option<usize>,
    mmio_max_32_bit: Option<usize>,
    mmio_64_bit: Option<usize>,
    mmio_max_64_bit: Option<usize>,
}

impl PciMemory {
    fn parse_from_fdt(fdt: fdt::Fdt) -> Option<PciMemory> {
        let nodes = fdt.find_compatible(COMPATIBLE)?;
        let memory = nodes.reg()?.next()?;

        let base_address = memory.starting_address as usize;
        let base_address_size = memory.size.unwrap_or(PCI_DEFAULT_MEM_SIZE);

        let mut mmio_32_bit = None;
        let mut mmio_max_32_bit = None;
        let mut mmio_64_bit = None;
        let mut mmio_max_64_bit = None;

        // https://www.devicetree.org/open-firmware/bindings/pci/pci-express.txt
        // we mostly map ranges here which will be used to allocate mem for devices. PIO is unsupported
        // on RISC-V and CONFIG space will not be used to allocate any memory, hence they are ignored
        for range in nodes.ranges()? {
            let hi = range.child_bus_address_hi;

            let space_code = (hi >> 24) & 0b11;

            match space_code {
                RANGE_MMIO_64_BIT => {
                    mmio_32_bit = Some(range.child_bus_address);
                    mmio_max_32_bit = Some(range.child_bus_address + range.size);
                }
                RANGE_MMIO_32_BIT => {
                    mmio_64_bit = Some(range.child_bus_address);
                    mmio_max_64_bit = Some(range.child_bus_address + range.size);
                }
                RANGE_CONFIG | RANGE_PIO => (/* PIO is not supported on RISC-V */),
                code => unreachable!("found code {code:#b} when expected in (inc)range 0b00-0b11"),
            };
        }

        Some(PciMemory {
            base_address,
            base_address_size,

            mmio_32_bit,
            mmio_max_32_bit,
            mmio_64_bit,
            mmio_max_64_bit,
        })
    }

    /// If you allocate what you dont need, I WILL spank you. I am not working on a deallocator for
    /// something so fucking basic
    pub fn allocate(&mut self, size: usize, is_64_bits: bool) -> Option<usize> {
        let (address_base, addr_max) = if is_64_bits {
            (self.mmio_64_bit?, self.mmio_max_64_bit?)
        } else {
            (self.mmio_32_bit?, self.mmio_max_32_bit?)
        };

        let alignment = size;

        let address = (address_base + alignment - 1) & !(alignment - 1);

        let next_addr = address + size;

        if next_addr < addr_max {
            self.mmio_64_bit = Some(next_addr);
            Some(address)
        } else {
            None
        }
    }

    fn map_memory(&self, mapper: &mut Mapper) {
        let base_mem_addr = self.base_address;
        let base_mem_pages = round_down_by(self.base_address_size, PAGE_SIZE) / PAGE_SIZE;

        mapper.map(
            base_mem_addr,
            base_mem_addr,
            Perms::READ_WRITE,
            base_mem_pages,
        );

        if let (Some(mmio_addr), Some(mmio_max)) = (self.mmio_64_bit, self.mmio_max_64_bit) {
            let mmio_size = mmio_max - mmio_addr;
            let mmio_pages = round_down_by(mmio_size, PAGE_SIZE) / PAGE_SIZE;

            mapper.map(mmio_addr, mmio_addr, Perms::READ_WRITE, mmio_pages);
        }

        if let (Some(mmio_addr), Some(mmio_max)) = (self.mmio_32_bit, self.mmio_max_32_bit) {
            let mmio_size = mmio_max - mmio_addr;
            let mmio_pages = round_down_by(mmio_size, PAGE_SIZE) / PAGE_SIZE;

            mapper.map(mmio_addr, mmio_addr, Perms::READ_WRITE, mmio_pages);
        }
    }
}

#[derive(Debug)]
#[repr(C, packed)]
struct Capabilities<T> {
    cap_id: u8,
    next_cap: u8,
    data: T,
}

/// Enumerate PCI devices. Returns a Vector (heap allocated)
fn enumerate_devices(ecam: Ecam) -> Vec<Device> {
    let mut devices = Vec::new();
    bruteforce_enumerate(ecam, &mut devices);
    devices
}

/// Not the best way, as we are just looping over all devices and busses. But it is good enough for
/// now, and it is not too inefficient as we do not really have a lot of devices in our VM
fn bruteforce_enumerate<T: Extend<Device>>(ecam: Ecam, list: &mut T) {
    for bus in 0..=255 {
        for device in 0..32 {
            for func in 0..8 {
                list.extend(ecam.get_device(bus, device, func));
            }
        }
    }
}
