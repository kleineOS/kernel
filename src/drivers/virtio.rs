use core::ops::RangeInclusive;

use alloc::vec::Vec;

use super::pci::Device;

const VENDOR_ID: u16 = 0x1AF4;
const DEVICE_ID_RANGE: RangeInclusive<u16> = 0x1000..=0x103F;

pub fn init(devices: &[Device]) {
    log::debug!("[DRIVER::VIRTIO] scanning for VirtIO devices");
    let devices = devices
        .iter()
        .filter(|device| device.vendor_id == VENDOR_ID)
        .copied()
        .inspect(|device| assert!(DEVICE_ID_RANGE.contains(&device.device_id)))
        .collect::<Vec<Device>>();

    for device in devices {
        log::debug!("[DRIVER::VIRTIO] VIRTIO DEVICE: {device}");
    }
}
