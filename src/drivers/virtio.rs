//! VirtIO General PCI driver
//! current version: 0.2-dev

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use super::DriverError;
use super::regcell::*;
use crate::systems::pci::{Device, PciMemory};

pub const ID_PAIR: (u16, u16) = (0x1af4, 0x1001);

pub fn init(device: Device, mem: &mut PciMemory) {
    log::info!("[VIRTIO] initialising VirtIO PCI driver");

    let config = match init_pci(&device, mem) {
        Ok(config) => config,
        Err(error) => {
            log::error!("[VIRTIO] driver init was a failure: {error}");
            return;
        }
    };

    if let Err(error) = config.boot() {
        log::error!("[VIRTIO] driver init was a failure: {error}");
        return;
    }

    unsafe {
        log::info!(
            "[VIRTOO] VirtIO device is now ready for I/O operations {:#x?}",
            *config.inner
        )
    };

    log::info!("[VIRTIO] driver init was a success!!");
}

fn init_pci(device: &Device, mem: &mut PciMemory) -> Result<VirtioPciCommonCfg, DriverError> {
    let mut cap = Vec::<CapData>::new();
    device.get_capabilities::<CapData, Vec<CapData>>(&mut cap);

    let bars: BTreeSet<u8> = cap
        .iter()
        .filter(|cap| cap.length > 0)
        .map(|cap| cap.bar)
        .collect();
    let bar_addrs = super::allocate_bar_addrs(bars, device, mem)?;

    let config: Option<&CapData> = cap.iter().find(|e| e.typ == CapDataType::Common);
    let data = config.ok_or(DriverError::OtherError(
        "CapData not found in device config",
    ))?;

    let address = bar_addrs.get(&data.bar).ok_or(DriverError::OtherError(
        "address for bar has not been allocated",
    ))?;
    let config = unsafe { VirtioPciCommonCfg::from_raw(address + data.offset as usize) };

    Ok(config)
}

struct VirtioPciCommonCfg {
    inner: *mut VirtioPciCommonCfgRaw,
}

impl VirtioPciCommonCfg {
    pub unsafe fn from_raw(addr: usize) -> Self {
        let inner = addr as *mut VirtioPciCommonCfgRaw;
        Self { inner }
    }

    // Page 59 of VirtIO spec v1.3
    // STEPS 1-8 (except 7, and some parts of 4) are all setup here
    pub fn boot(&self) -> Result<(), DriverError> {
        let inner = unsafe { &*self.inner };

        // STEP 1
        inner.device_status.set(DeviceStatus::RESET);

        // STEP 2
        inner.device_status.set(DeviceStatus::ACKNOWLEDGE);

        // STEP 3
        let status = inner.device_status.get();
        inner.device_status.set(status | DeviceStatus::DRIVER);

        // STEP 4
        // we need to get 64 bits of device features here
        inner.device_feature_select.set(0);
        let device_feature_le = inner.device_feature.get();
        inner.device_feature_select.set(1);
        let device_feature_hi = inner.device_feature.get();

        // log::debug!("VirtIO device features {device_feature_hi:#x} {device_feature_le:#x}");

        inner.driver_feature_select.set(0);
        inner.driver_feature.set(device_feature_le);
        inner.driver_feature_select.set(1);
        inner.driver_feature.set(device_feature_hi);

        // STEP 5
        let status = inner.device_status.get();
        inner.device_status.set(status | DeviceStatus::FEATURES_OK);

        // STEP 6
        let status = inner.device_status.get();
        if !status.contains(DeviceStatus::FEATURES_OK) {
            inner.device_status.set(status | DeviceStatus::FAILED);
            return Err(DriverError::OtherError("device is not ok"));
        }

        // TODO: STEP 7

        // STEP 8
        let status = inner.device_status.get();
        inner.device_status.set(status | DeviceStatus::DRIVER_OK);

        Ok(())
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    struct DeviceStatus: u8 {
        const RESET = 0;
        const ACKNOWLEDGE = 1;
        const DRIVER = 2;
        const DRIVER_OK = 4;
        const FEATURES_OK = 8;
        const DEVICE_NEEDS_RESET = 64;
        const FAILED = 128;
    }
}

#[repr(C)]
#[derive(Debug)]
struct VirtioPciCommonCfgRaw {
    /* About the whole device. */
    device_feature_select: RegCell<u32, RW>,
    device_feature: RegCell<u32>,
    driver_feature_select: RegCell<u32, RW>,
    driver_feature: RegCell<u32, RW>,
    config_msix_vector: RegCell<u16, RW>,
    num_queues: RegCell<u16>,
    device_status: RegCell<DeviceStatus, RW>,
    config_generation: RegCell<u8>,

    /* About a specific virtqueue. */
    queue_select: RegCell<u16, RW>,
    queue_msix_vector: RegCell<u16, RW>,
    queue_enable: RegCell<u16, RW>,
    queue_notify_off: RegCell<u16>,
    queue_desc: RegCell<u64, RW>,
    queue_driver: RegCell<u64, RW>,
    queue_device: RegCell<u64, RW>,
    queue_notif_config_data: RegCell<u16>,
    queue_reset: RegCell<u16, RW>,

    /* About the administration virtqueue. */
    admin_queue_index: RegCell<u16>,
    admin_queue_num: RegCell<u16>,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(unused)]
enum CapDataType {
    Common = 1,
    Notify = 2,
    Isr = 3,
    Device = 4,
    Pci = 5,
    SharedMemory = 8,
    Vendor = 9,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct CapData {
    len: u8,
    typ: CapDataType,
    bar: u8,
    id: u8,
    _padding: [u8; 2],
    offset: u32,
    length: u32,
}
