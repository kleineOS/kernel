//! VirtIO Block Device driver
//! TODO: I will hardcode some values regarding PCIe Bus and Device number
//! - Will need to remove the hardcoded values later
//! - Allow a mechanism for drivers to request to be the "owners" of some PCIe devices

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

    fn init(&self, header: PcieEcamHeader, _: PcieEcam) {
        log::debug!("virtio {header:#x?}");
    }
}

// pub fn init(stub: PcieEcamHeader, _: PcieEcam) {
//     let bus_hardcode = 0;
//     let device_hardcode = 1;
//
//     let device = ecam
//         .get_common_dev_info(bus_hardcode, device_hardcode)
//         .unwrap();
//
//     assert_eq!(device.header_type, 0x0);
//     assert_eq!(device.vendor_id, VENDOR_ID);
//     assert!(DEVICE_ID_RANGE.contains(&device.device_id));
//
//     // read the subsystem id
//     let cache = ecam.read_register(bus_hardcode, device_hardcode, 0xb);
//     let subsystem_id = (cache >> 16) as u16;
//     assert_eq!(subsystem_id, 0x2, "block device needs to be subsystem 0x2");
// }
