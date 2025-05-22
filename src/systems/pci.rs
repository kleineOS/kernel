//! PCIe subsystem for the kleineOS kernel
//! current version: 0.2-dev

const COMPATIBLE: &[&str] = &["pci-host-ecam-generic"];
const PCI_DEFAULT_MEM_SIZE: usize = crate::PAGE_SIZE;

pub struct PciSubsystem {
    mem: PcieMemory,
}

impl PciSubsystem {
    pub fn init(fdt: fdt::Fdt) -> Option<Self> {
        let mem = parse_fdt(fdt)?;

        Some(Self { mem })
    }

    pub fn allocate_64bit(size: usize) {
        todo!()
    }
}

#[derive(Debug)]
#[allow(unused)]
struct PcieMemory {
    base_address: usize,
    base_address_size: usize,

    mmio_32_bit: Option<usize>,
    mmio_size_32_bit: Option<usize>,
    mmio_64_bit: Option<usize>,
    mmio_size_64_bit: Option<usize>,
}

fn parse_fdt(fdt: fdt::Fdt) -> Option<PcieMemory> {
    let nodes = fdt.find_compatible(COMPATIBLE)?;
    let memory = nodes.reg()?.next()?;

    let base_address = memory.starting_address as usize;
    let base_address_size = memory.size.unwrap_or(PCI_DEFAULT_MEM_SIZE);

    let mut mmio_32_bit = None;
    let mut mmio_size_32_bit = None;
    let mut mmio_64_bit = None;
    let mut mmio_size_64_bit = None;

    // https://www.devicetree.org/open-firmware/bindings/pci/pci-express.txt
    for range in nodes.ranges()? {
        let hi = range.child_bus_address_hi;

        let space_code = (hi >> 24) & 0b11;

        match space_code {
            0b10 => {
                mmio_32_bit = Some(range.child_bus_address);
                mmio_size_32_bit = Some(range.size);
            }
            0b11 => {
                mmio_64_bit = Some(range.child_bus_address);
                mmio_size_64_bit = Some(range.size);
            }
            0b00 | 0b01 => (/* PIO is not supported on RISC-V */),
            code => unreachable!("found code {code:#b} when expected in (inc)range 0b00-0b11"),
        };
    }

    Some(PcieMemory {
        base_address,
        base_address_size,

        mmio_32_bit,
        mmio_size_32_bit,
        mmio_64_bit,
        mmio_size_64_bit,
    })
}

// dtb parsing
// lets take the ranges field from the qemu-virt machine
// ranges = <0x1000000 0x00 0x00       0x00 0x3000000  0x00 0x10000
//           0x2000000 0x00 0x40000000 0x00 0x40000000 0x00 0x40000000
//           0x3000000 0x04 0x00       0x04 0x00       0x04 0x00>;
//
// range#1: 0x1000000 0x00 0x00 // high mid low
//          0x00 0x3000000      // parent_high parent_low
//          0x00 0x10000        // size_high size_low
//
// range#2: 0x2000000 0x00 0x40000000   // high mid low
//          0x00 0x40000000             // parent_high parent_low
//          0x00 0x40000000             // size_high size_low
//
// range#3: 0x3000000 0x04 0x00 // high mid low
//          0x04 0x00           // parent_high parent_low
//          0x04 0x00           // size_high size_low
