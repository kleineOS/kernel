//! PCIe subsystem for the kleineOS kernel
//! current version: 0.2-dev

const COMPATIBLE: &[&str] = &["pci-host-ecam-generic"];
const PCI_DEFAULT_MEM_SIZE: usize = crate::PAGE_SIZE;

pub struct PciSubsystem {}

impl PciSubsystem {
    pub fn init(fdt: fdt::Fdt) {
        log::trace!("{:#x?}", parse_fdt(fdt));
    }
}

#[derive(Debug)]
struct PcieMemory {
    base_address: usize,
    base_address_size: usize,
}

fn parse_fdt(fdt: fdt::Fdt) -> Option<PcieMemory> {
    let nodes = fdt.find_compatible(COMPATIBLE)?;
    let memory = nodes.reg()?.next()?;

    let base_address = memory.starting_address as usize;
    let base_address_size = memory.size.unwrap_or(PCI_DEFAULT_MEM_SIZE);

    for (i, range) in nodes.ranges()?.enumerate() {
        log::trace!("#{i} :: {range:#x?}");
    }

    Some(PcieMemory {
        base_address,
        base_address_size,
    })
}
