//! VirtIO General PCI driver
//! current version: 0.2-dev

mod block;
mod virtqueue;

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use block::BlkConfig;
use virtqueue::VirtQueue;

use super::DriverError;
use super::regcell::*;
use crate::systems::pci::{Device, PciMemory};

// ID_PAIR for a virtio block device, I will add more support once this is done
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
    }
}

#[allow(unused)]
#[derive(Debug)]
struct Data {
    common: CapData,
    notify: CapData,
    isr: CapData,
    device: CapData,
    pci: CapData,
}

macro_rules! require_cap {
    ($cap:expr, $name:literal) => {
        match $cap {
            Some(cap) => *cap,
            None => {
                log::error!("Missing {} capability", $name);
                return None;
            }
        }
    };
}

fn read_cap_data(cap: &[CapData]) -> Option<Data> {
    let mut common = None;
    let mut notify = None;
    let mut isr = None;
    let mut device = None;
    let mut pci = None;

    for cap in cap {
        match cap.typ {
            CapDataType::Common => common = Some(cap),
            CapDataType::Notify => notify = Some(cap),
            CapDataType::Isr => isr = Some(cap),
            CapDataType::Device => device = Some(cap),
            CapDataType::Pci => pci = Some(cap),
            _ => {
                core::hint::cold_path();
                log::warn!("Unknown capability type for a VirtIO device: {:?}", cap.typ);
                continue;
            }
        }
    }

    let common = require_cap!(common, "common");
    let notify = require_cap!(notify, "notify");
    let isr = require_cap!(isr, "isr");
    let device = require_cap!(device, "device");
    let pci = require_cap!(pci, "pci");

    Some(Data {
        common,
        notify,
        isr,
        device,
        pci,
    })
}

fn init_pci(device: &Device, mem: &mut PciMemory) -> Result<VirtioPciCommonCfg, DriverError> {
    let mut cap = Vec::<CapData>::new();
    device.get_capabilities::<CapData, Vec<CapData>>(&mut cap);

    let cap_data = read_cap_data(&cap).ok_or(DriverError::OtherError(
        "Device capability list is incomplete",
    ))?;

    let bars: BTreeSet<u8> = cap
        .iter()
        // bar 0 is going to be for PIO, so we skip it on RISCV TODO-ARCH-RISCV
        .filter_map(|cap| if cap.bar > 0 { Some(cap.bar) } else { None })
        .collect();

    let bar_addrs = super::allocate_bar_addrs(bars, device, mem)?;

    let data = cap_data.common;
    let address = bar_addrs.get(&data.bar).ok_or(DriverError::OtherError(
        "address for bar has not been allocated",
    ))?;
    let config = unsafe { VirtioPciCommonCfg::from_raw(address + data.offset as usize) };

    // device data stuff
    let blk_cfg_data = cap_data.device;
    let _blk_config = unsafe { BlkConfig::from_raw(address + blk_cfg_data.offset as usize) };
    // unsafe { log::info!("[VIRTIO] BLOCK DEVICE CONFIG: {:#x?}", *blk_config.inner) };

    Ok(config)
}

struct VirtioPciCommonCfg {
    common_raw: *mut VirtioPciCommonCfgRaw,
}

impl VirtioPciCommonCfg {
    pub unsafe fn from_raw(addr: usize) -> Self {
        let inner = addr as *mut VirtioPciCommonCfgRaw;
        Self { common_raw: inner }
    }

    // Page 59 of VirtIO spec v1.3
    // STEPS 1-8 (except 7, and some parts of 4) are all setup here
    pub fn boot(&self) -> Result<(), DriverError> {
        let inner = unsafe { &*self.common_raw };

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
        let feats = BlockDevFeatures::from_bits_truncate(device_feature_le);
        // log::info!(
        //     "HELLO WORLD {:#x?}",
        //     BlockDevFeatures::from_bits_truncate(device_feature_le)
        // );
        // inner.device_feature_select.set(1);
        // let device_feature_hi = inner.device_feature.get();
        // log::info!(
        //     "HELLO WORLD {:#x?}",
        //     BlockDevFeatures::from_bits_truncate(device_feature_hi)
        // );

        // log::debug!("VirtIO device features {device_feature_hi:#x} {device_feature_le:#x}");

        inner.driver_feature_select.set(0);
        inner.driver_feature.set(feats.bits());
        // inner.driver_feature_select.set(1);
        // inner.driver_feature.set(device_feature_hi);

        // STEP 5
        let status = inner.device_status.get();
        inner.device_status.set(status | DeviceStatus::FEATURES_OK);

        // STEP 6
        let status = inner.device_status.get();
        if !status.contains(DeviceStatus::FEATURES_OK) {
            inner.device_status.set(status | DeviceStatus::FAILED);
            return Err(DriverError::OtherError("device is not ok"));
        }

        // STEP 7
        let virtqueues = self.probe_virtqueues();
        VirtQueue::init(virtqueues);

        // STEP 8
        let status = inner.device_status.get();
        inner.device_status.set(status | DeviceStatus::DRIVER_OK);

        Ok(())
    }

    fn probe_virtqueues(&self) -> BTreeMap<u16, u16> {
        let inner = unsafe { &*self.common_raw };

        let max_virtqueues = inner.num_queues.get();
        let mut map = BTreeMap::new();

        for queue in 0..max_virtqueues {
            inner.queue_select.set(queue);
            let size = inner.queue_size.get();

            if size == 0 {
                break;
            }

            map.insert(queue, size);
        }

        map
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    struct DeviceStatus: u8 {
        const RESET = 0;
        const ACKNOWLEDGE = 1;  // bit: 0
        const DRIVER = 2;       // bit: 1
        const DRIVER_OK = 4;    // bit: 2
        const FEATURES_OK = 8;  // bit: 3
        const DEVICE_NEEDS_RESET = 64; // bit: 6
        const FAILED = 128;     // bit: 7
    }

    #[derive(Debug)]
    struct BlockDevFeatures: u32 {
        const SIZE_MAX = 1 << 2;
        const SEG_MAX = 1 << 3;
        const GEOMETRY = 1 << 5;
        const RO = 1 << 6;
        const BLK_SIZE = 1 << 7;
        const FLUSH = 1 << 10;
        const TOPOLOGY = 1 << 11;
        const CONFIG_WCE = 1 << 12;
        const CONFIG_MQ = 1 << 13;
        const DISCARD = 1 << 14;
        const WRITE_ZEROES = 1 << 15;
        const LIFETIME = 1 << 16;
        const SECURE_ERASE = 1 << 17;
        const ZONED = 1 << 18;
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
    queue_size: RegCell<u16, RW>,
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
