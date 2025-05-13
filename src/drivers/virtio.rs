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

use crate::pci::{MoreDevInfo, PciDeviceInit, PcieEcam, PcieEcamHeader};

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
        log::debug!("initialising VirtIO Block Device driver");

        let bus = header.bus_nr;
        let device = header.device_nr;

        let dev_info = match header.get_more_dev_info(ecam) {
            MoreDevInfo::GeneralDev(info) => info,
            other => panic!("expected general device but found {other:?}"),
        };
        assert_eq!(dev_info.subsystem_id, 0x2, "block device should be 0x2");
        assert!(header.status_capabilities_list());

        let cap_base = dev_info.capabilities_ptr;

        let id = ecam.read_word(bus, device, 0, cap_base) as u8;
        log::debug!("{id:#x}");
        let next_offset = ecam.read_word(bus, device, 0, cap_base + 4) as u8;
        log::debug!("{next_offset:#x}");

        // log::trace!("{header:#x?}");
        // log::trace!("{dev_info:#x?}");
    }
}

// fn find_mmio(fdt: fdt::Fdt) {
//     let nodes = fdt.find_compatible(&["virtio,mmio"]).unwrap();
// }
