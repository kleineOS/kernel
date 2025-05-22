//! VirtIO Block Device driver
//! current version: 0.1-dev
//!
//! Based on https://docs.oasis-open.org/virtio/virtio/v1.3/virtio-v1.3.pdf
//! F\*\*\* PCI-SIG for making the spec a 5000USD annual subscription, I should just subscribe to
//! some AI BS instead and make 15 SAAS apps with the same budget.

use crate::pci::*;

#[derive(Debug)]
struct PostInitData {
    dev_info: GeneralDevInfo,
}

impl PostInitData {
    /// generate this struct using the [PcieEcamHeader] and the [GeneralDevInfo] structs
    pub fn generate(dev_info: GeneralDevInfo) -> Self {
        Self { dev_info }
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
        let bar_index = cap.bar;
        let bar_register = BAR_BASE_REG + bar_index;

        // we first read the size of the BAR
        let size = get_bar_size(ecam, bar_register);

        // then we allocate an memory that is aligned to the size of the BAR
        let address_base = 0x40000000; // read from dtb
        let alignment = size;
        let address = (address_base + alignment - 1) & !(alignment - 1);

        // in get_bar_size, we assert that bar type is 0x2 (64-bit)
        // so we do have to split our address into high and low and assign it properly
        ecam.write_register(bar_register, address);

        let new_lo = ecam.read_register(bar_register);
        log::trace!("ADDRESS={new_lo:#x}");

        ecam.write_word(0x04, 0b10000000010);
        let cmd = ecam.read_word(0x04) as u16;
        log::info!("Command Register = {cmd:#b}");

        let config = address as *const VirtioPciCommonCfg;
        unsafe { log::info!("config={:#x?}", *config) };
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
        log::info!("{header:#x?}");

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

        let post_init_data = PostInitData::generate(dev_info);
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
