//! VirtIO Block Device driver
//! current version: 0.1-dev
//!
//! Based on https://docs.oasis-open.org/virtio/virtio/v1.3/virtio-v1.3.pdf
//!
//! The following bits need to be set to initialise the device
//! - ACKNOWLEDGE   [1]     Ack from our OS indicating that we found the device
//! - DRIVER        [2]     Indicates our OS knows how to drive the device
//! - FEATURES_OK   [8]
//! - DRIVER_OK     [4]
//! - DEVICE_NEEDS_RESET [64]

use core::{
    alloc::Layout,
    ptr::{read_volatile, write_volatile},
};

use crate::pci::{
    self, GeneralDevInfo, MoreDevInfo, PciDeviceInit, PcieEcam, PcieEcamHeader, VirtioPciCap,
    VirtioPciCapCfg,
};

#[derive(Debug)]
struct PostInitData {
    dev_info: GeneralDevInfo,
    header: PcieEcamHeader,
}

impl PostInitData {
    /// generate this struct using the [PcieEcamHeader] and the [GeneralDevInfo] structs
    pub fn generate(dev_info: GeneralDevInfo, header: PcieEcamHeader) -> Self {
        Self { dev_info, header }
    }
}

pub struct BlkDriver {
    post_init_data: Option<PostInitData>,
}

impl BlkDriver {
    pub fn new() -> Self {
        let post_init_data = None;
        BlkDriver { post_init_data }
    }

    fn get_init_data(&self) -> Result<&PostInitData, ()> {
        self.post_init_data.as_ref().ok_or(())
    }

    fn common_cfg(&self, ecam: PcieEcam, base_addr: usize, cap: VirtioPciCap) {
        let init_data = self.get_init_data().expect("device is not initialised");
        let bar_index = cap.bar as usize;

        log::info!("{:#x?}", init_data.dev_info);

        let size = hailmary(base_addr + 0x10 + (bar_index * 4));
        let alloc_layout = Layout::from_size_align(size as usize, size as usize).unwrap();
        let address = unsafe { alloc::alloc::alloc(alloc_layout) };
        hailmary1(base_addr + 0x10 + (bar_index * 4), address as u32);

        let value = unsafe { read_volatile((base_addr + 0x10 + (bar_index * 4)) as *const u32) };
        log::info!("{value:#x}");

        let cmd = ecam.read_word(0, 1, 0, 0x04);
        ecam.write_word(0, 1, 0, 0x04, cmd | 0x02 | 0x04);

        // I put this here so I can go and check the address in gdb
        loop {}
    }

    fn enumerate_capabilities(&self, ecam: PcieEcam, base_addr: usize) {
        let init_data = self.get_init_data().expect("device is not initialised");
        let offset = init_data.dev_info.capabilities_ptr as usize;

        let capabilities = pci::enumerate_capabilities(base_addr, offset);
        let mut cap_iter = capabilities.into_iter();

        while let Some(Some(cap)) = cap_iter.next() {
            match cap.cfg_type {
                VirtioPciCapCfg::Common => self.common_cfg(ecam, base_addr, cap),
                cfg_type => log::trace!("TODO: {cfg_type:?} CAPABILITY CONFIG"),
            }
        }
    }
}

impl PciDeviceInit for BlkDriver {
    fn id_pair(&self) -> (u16, u16) {
        (0x1AF4, 0x1001)
    }

    fn init(&mut self, header: PcieEcamHeader, ecam: PcieEcam, _fdt: fdt::Fdt) {
        log::info!("Initialising VirtIO Block Device driver");

        let bus = header.bus_nr;
        let device = header.device_nr;

        let dev_info = match header.get_more_dev_info(ecam) {
            MoreDevInfo::GeneralDev(info) => info,
            other => panic!("expected general device but found {other:?}"),
        };

        // these things are always going to be valid for our use case, but it is better to hard
        // code the invariants as asserts. DO NOT REMOVE THESE
        assert_eq!(dev_info.subsystem_id, 0x2, "block device should be 0x2");
        assert!(header.status_capabilities_list());

        let post_init_data = PostInitData::generate(dev_info, header);
        self.post_init_data = Some(post_init_data);

        let base_addr = ecam.address(bus, device, 0);
        self.enumerate_capabilities(ecam, base_addr);
    }
}

fn hailmary(bar_addr: usize) -> u32 {
    let bar_addr = bar_addr as *mut u32;

    let original = unsafe { read_volatile(bar_addr) };

    unsafe { write_volatile(bar_addr, u32::MAX) };
    let register = unsafe { read_volatile(bar_addr) };

    unsafe { core::ptr::write_volatile(bar_addr, original) };

    let is_pio = (register & 1) != 0;
    assert!(!is_pio, "RISC-V does not support PIO");

    let bar_type = (register >> 1) & 0b11;
    log::info!("BAR TYPE: {bar_type:#x}");

    // the first few bits are for conveying info to us, the os
    let size = !(register & 0xFFFFFFF0) + 1;
    log::info!("{size:#x}");
    size
}

fn hailmary1(bar_addr: usize, address: u32) {
    let bar_addr = bar_addr as *mut u32;
    unsafe { write_volatile(bar_addr, address) };
}

// fn find_mmio(fdt: fdt::Fdt) {
//     let nodes = fdt.find_compatible(&["virtio,mmio"]).unwrap();
// }
