//! VirtIO Block Device driver
//! current version: 0.1-dev
//!
//! Based on https://docs.oasis-open.org/virtio/virtio/v1.3/virtio-v1.3.pdf
//! F\*\*\* PCI-SIG for making the spec a 5000USD annual subscription, I should just subscribe to
//! some AI BS instead and make 15 SAAS apps with the same budget.

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
        todo!("read common config register")
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

fn get_bar_size(bar_addr: usize) -> u32 {
    let bar_addr = bar_addr as *mut u32;

    let original = unsafe { read_volatile(bar_addr) };

    unsafe { write_volatile(bar_addr, u32::MAX) };
    let register = unsafe { read_volatile(bar_addr) };

    unsafe { core::ptr::write_volatile(bar_addr, original) };

    let is_pio = (register & 1) != 0;
    assert!(!is_pio, "RISC-V does not support PIO");

    let _bar_type = (register >> 1) & 0b11;

    // the first few bits are for conveying info to us, the os
    !(register & 0xFFFFFFF0) + 1
}

fn assign_bar(ecam: PcieEcam, bus: u8, device: u8, function: u8, bar_index: u8, mmio_base: usize) {
    let bar_offset = 0x10 + (bar_index * 4);
    let lo = (mmio_base as u32) & 0xFFFFFFF0;

    ecam.write_word(bus, device, function, bar_offset, lo);
    ecam.write_word(bus, device, function, bar_offset + 4, 0);
}

#[repr(C)]
#[derive(Debug)]
struct VirtioPciCommonCfg {
    /* About the whole device. */
    device_feature_select: u32, // RW
    device_feature: u32,        // RO
    driver_feature_select: u32, // RW
    driver_feature: u32,        // RW
    config_msix_vector: u16,    // RW
    num_queues: u16,            // RO
    device_status: u8,          // RW
    config_generation: u8,      // RO

    /* About a specific virtqueue. */
    queue_select: u16,            // RW
    queue_msix_vector: u16,       // RW
    queue_enable: u16,            // RW
    queue_notify_off: u16,        // RO
    queue_desc: u64,              // RW
    queue_driver: u64,            // RW
    queue_device: u64,            // RW
    queue_notif_config_data: u16, // RO
    queue_reset: u16,             // RW

    /* About the administration virtqueue. */
    admin_queue_index: u16, // RO
    admin_queue_num: u16,   // RO
}

// fn find_mmio(fdt: fdt::Fdt) {
//     let nodes = fdt.find_compatible(&["virtio,mmio"]).unwrap();
// }
