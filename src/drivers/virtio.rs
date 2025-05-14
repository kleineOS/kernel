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

use crate::pci::{self, MoreDevInfo, PciDeviceInit, PcieEcam, PcieEcamHeader, VirtioPciCapCfg};

pub struct BlkDriver {}

impl BlkDriver {
    pub fn new() -> Self {
        BlkDriver {}
    }
}

impl PciDeviceInit for BlkDriver {
    fn id_pair(&self) -> (u16, u16) {
        (0x1AF4, 0x1001)
    }

    fn init(&self, header: PcieEcamHeader, ecam: PcieEcam, _fdt: fdt::Fdt) {
        log::info!("Initialising VirtIO Block Device driver");

        let bus = header.bus_nr;
        let device = header.device_nr;

        let dev_info = match header.get_more_dev_info(ecam) {
            MoreDevInfo::GeneralDev(info) => info,
            other => panic!("expected general device but found {other:?}"),
        };
        assert_eq!(dev_info.subsystem_id, 0x2, "block device should be 0x2");
        assert!(header.status_capabilities_list());

        let base_addr = ecam.address(bus, device, 0);
        let capabilities =
            pci::enumerate_capabilities(base_addr, dev_info.capabilities_ptr as usize);
        let mut iter = capabilities.into_iter();

        let mut common_cap = None;
        while let Some(Some(cap)) = iter.next() {
            log::trace!("{cap:x?}");
            if matches!(cap.cfg_type, VirtioPciCapCfg::Common) {
                common_cap = Some(cap);
            }
        }

        let common_cap = common_cap.unwrap();
        let bar_offset = dev_info.base_addrs[common_cap.bar as usize];
        let bar_addr = base_addr + (bar_offset + common_cap.offset_le.to_be()) as usize;

        // log::trace!("{common_cap:x?}");
        log::trace!("{header:x?}");
        log::trace!("{dev_info:x?}");
        log::trace!("base_addr={bar_addr:#x}");
    }
}

// fn find_mmio(fdt: fdt::Fdt) {
//     let nodes = fdt.find_compatible(&["virtio,mmio"]).unwrap();
// }
