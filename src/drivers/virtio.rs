//! VirtIO Block Device driver

use crate::pci::{PciDeviceInit, PcieEcam, PcieEcamHeader};

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

    fn init(&self, header: PcieEcamHeader, ecam: PcieEcam) {
        log::debug!("initialising VirtIO Block Device driver");

        let dev_info = match header.get_more_dev_info(ecam) {
            crate::pci::MoreDevInfo::GeneralDev(info) => info,
            other => panic!("expected general device but found {other:?}"),
        };
        assert_eq!(dev_info.subsystem_id, 0x2, "block device should be 0x2");

        log::trace!("{dev_info:?}");
    }
}
