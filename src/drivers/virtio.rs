//! VirtIO Block Device driver
//! current version: 0.1-dev
//!
//! Based on https://docs.oasis-open.org/virtio/virtio/v1.3/virtio-v1.3.pdf
//! F\*\*\* PCI-SIG for making the spec a 5000USD annual subscription, I should just subscribe to
//! some AI BS instead and make 15 SAAS apps with the same budget.

use core::alloc::Layout;

use crate::pci::*;

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

    fn common_cfg(&self, ecam: PcieEcamLocked, cap: VirtioPciCap) {
        crate::println!("\n\n\n");

        let bar_index = cap.bar;
        let bar_register = BAR_BASE_REG + bar_index;

        // we first read the size of the BAR
        let size = get_bar_size(ecam, bar_register);

        // then we allocate an memory that is aligned to the size of the BAR
        let alloc_layout = Layout::from_size_align(size as usize, size as usize).unwrap();
        let address = unsafe { alloc::alloc::alloc(alloc_layout) };

        let address_num = address as usize;
        let addr_lo = (address_num & 0xFFFF_FFFF) as u32;
        let addr_hi = (address_num >> 32) as u32;

        let original = ecam.read_register(bar_register);

        // in get_bar_size, we assert that bar type is 0x2 (64-bit)
        // so we do have to split our address into high and low and assign it properly
        ecam.write_register(bar_register, addr_lo);
        ecam.write_register(bar_register + 1, addr_hi);

        let new_lo = ecam.read_register(bar_register);
        let new_hi = ecam.read_register(bar_register + 1);
        assert_eq!(new_hi, 0, "deal with doing things the proper way later");

        log::info!("size={size:#x} address={address_num:#x}");
        log::info!("original={original:#x} -> lo={new_lo:#x}");

        let config = address_num as *const VirtioPciCommonCfg;
        unsafe { log::info!("config={:#x?}", *config) };

        // sanity check
        let bar_lo = ecam.read_register(bar_register);
        let bar_hi = ecam.read_register(bar_register + 1);
        let bar_combined = ((bar_hi as u64) << 32) | (bar_lo as u64);

        // Mask off the bottom 4 bits — only upper bits are the base address
        let phys_addr = bar_combined & 0xFFFF_FFF0;
        assert_eq!(phys_addr as usize, address_num);

        crate::println!("\n\n\n");
        todo!("read common config register")
    }

    fn enumerate_capabilities(&self, ecam: PcieEcamLocked, base_addr: usize) {
        let init_data = self.get_init_data().expect("device is not initialised");
        let offset = init_data.dev_info.capabilities_ptr as usize;

        let capabilities = enumerate_capabilities(base_addr, offset);
        let mut cap_iter = capabilities.into_iter();

        while let Some(Some(cap)) = cap_iter.next() {
            match cap.cfg_type {
                VirtioPciCapCfg::Common => self.common_cfg(ecam, cap),
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

        let ecam_locked = ecam.get_locked(bus, device);

        let base_addr = ecam.address(bus, device, 0);
        self.enumerate_capabilities(ecam_locked, base_addr);
    }
}

fn get_bar_size(ecam: PcieEcamLocked, register: u8) -> u32 {
    let original = ecam.read_register(register);

    ecam.write_register(register, u32::MAX);
    let new_value = ecam.read_register(register);

    ecam.write_register(register, original);

    let is_pio = (new_value & 1) != 0;
    assert!(!is_pio, "RISC-V does not support PIO");

    let _bar_type = (new_value >> 1) & 0b11;

    // the first few bits are for conveying info to us, the os
    !(new_value & 0xFFFFFFF0) + 1
}

#[repr(C, packed)]
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
