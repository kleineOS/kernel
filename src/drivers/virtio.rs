//! VirtIO Block Device driver
//! current version: 0.2-dev

use super::DriverError;
use crate::systems::pci::{Device, HeaderType};

pub const ID_PAIR: (u16, u16) = (0x1af4, 0x1001);

pub fn init(device: Device) {
    log::info!("[VIRTIO] initialising block device driver");
    match init_driver(&device) {
        Ok(_) => log::info!("[VIRTIO] driver init was a success!!"),
        Err(error) => log::error!("[VIRTIO] driver init was a failure: {error}"),
    }
}

fn init_driver(device: &Device) -> Result<(), DriverError> {
    let devinfo = DeviceInfo::get_general_info(device)?;

    device.get_capabilities();

    devinfo.read_bars();

    Ok(())
}

/// I dont know if this needs to be here or in the PCI subsystem, will sort it out when I have one
/// more driver. I will move all the shared logic to the PCI subsystem
struct DeviceInfo<'a> {
    device: &'a Device,
}

impl<'a> DeviceInfo<'a> {
    fn get_general_info(device: &'a Device) -> Result<Self, DriverError> {
        match device.header_type() {
            HeaderType::GeneralDevice => (),
            HeaderType::Pci2Pci => {
                let reason = "Expected GeneralDevice but got PCI to PCI bridge";
                return Err(DriverError::InvalidDevice { reason });
            }
            HeaderType::Pci2Cardbus => {
                let reason = "Expected GeneralDevice but got PCI to Cardbus bridge";
                return Err(DriverError::InvalidDevice { reason });
            }
        };

        Ok(Self { device })
    }

    #[allow(unused)]
    fn read_bars(&self) {
        let ecam = &self.device.ecam;
        ecam.write::<u32>(0x20, u32::MAX);
        let bars = ecam.read::<[u32; 6]>(0x10);

        for bar in bars {
            let is_pio = bar & 0b1 > 0; // bit 0
            let typ = (bar >> 1) & 0b11; // bits 2:1
            let is_prefetchable = (bar >> 3) & 0b1 > 0; // bits 2:1

            let addr = bar & 0xFFFFFFF0;

            // log::info!(
            //     "BAR[{addr:#x}]: is_pio={is_pio}, typ={typ}, is_prefetchable={is_prefetchable}",
            // );
        }

        //log::info!("BARS={bars:#018x?}");
    }
}
