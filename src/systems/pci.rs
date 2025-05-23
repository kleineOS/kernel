//! PCIe subsystem for the kleineOS kernel
//! current version: 0.2-dev
#![allow(unused)]

use alloc::vec::Vec;

use crate::vmem::{Mapper, Perms};
use crate::{PAGE_SIZE, round_down_by};

const COMPATIBLE: &[&str] = &["pci-host-ecam-generic"];
const PCI_DEFAULT_MEM_SIZE: usize = PAGE_SIZE;

const RANGE_MMIO_64_BIT: u32 = 0b10;
const RANGE_MMIO_32_BIT: u32 = 0b11;
const RANGE_PIO: u32 = 0b01;
const RANGE_CONFIG: u32 = 0b00;

pub struct PciSubsystem {
    mem: PcieMemory,
    ecam: Ecam,
    devices: Vec<()>,
}

struct Ecam {}

impl Ecam {
    fn init(config_base_addr: usize) -> Self {
        Self {}
    }
}

impl PciSubsystem {
    pub fn init(fdt: fdt::Fdt, mapper: &mut Mapper) -> Option<Self> {
        let mem = PcieMemory::parse_from_fdt(fdt)?;
        mem.map_memory(mapper);

        let ecam = Ecam::init(mem.base_address);
        let devices = enumerate_devices();

        Some(Self { mem, ecam, devices })
    }
}

#[derive(Debug)]
#[allow(unused)]
pub struct PcieMemory {
    base_address: usize,
    base_address_size: usize,

    mmio_32_bit: Option<usize>,
    mmio_max_32_bit: Option<usize>,
    mmio_64_bit: Option<usize>,
    mmio_max_64_bit: Option<usize>,
}

impl PcieMemory {
    fn parse_from_fdt(fdt: fdt::Fdt) -> Option<PcieMemory> {
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

        Some(PcieMemory {
            base_address,
            base_address_size,

            mmio_32_bit,
            mmio_max_32_bit,
            mmio_64_bit,
            mmio_max_64_bit,
        })
    }
    pub fn allocate_64bit(&mut self, size: usize) -> Option<usize> {
        let addr = self.mmio_64_bit?;
        let addr_max = self.mmio_max_64_bit?;

        let next_addr = addr + size;

        if next_addr < addr_max {
            self.mmio_64_bit = Some(next_addr);
            Some(addr)
        } else {
            None
        }
    }

    pub fn allocate_32bit(&mut self, size: usize) -> Option<usize> {
        let addr = self.mmio_32_bit?;
        let addr_max = self.mmio_max_32_bit?;

        let next_addr = addr + size;

        if next_addr < addr_max {
            self.mmio_32_bit = Some(next_addr);
            Some(addr)
        } else {
            None
        }
    }

    pub fn map_memory(&self, mapper: &mut Mapper) {
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

/// Enumerate PCI devices. Returns a Vector (heap allocated)
fn enumerate_devices() -> Vec<()> {
    let devices = Vec::new();
    bruteforce_enumerate();
    devices
}

/// Not the best way, as we are just looping over all devices and busses. But it is good enough for
/// now, and it is not too inefficient as we do not really have a lot of devices in our VM
fn bruteforce_enumerate() {}
