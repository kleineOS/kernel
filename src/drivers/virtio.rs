//! VirtIO Block Device driver
//! current version: 0.2-dev

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use super::DriverError;
use super::regcell::*;
use crate::systems::pci::{Device, PciMemory};

pub const ID_PAIR: (u16, u16) = (0x1af4, 0x1001);

pub fn init(device: Device, mem: &mut PciMemory) {
    log::info!("[VIRTIO] initialising block device driver");

    let config = match init_pci(&device, mem) {
        Ok(config) => config,
        Err(error) => {
            log::error!("[VIRTIO] driver init was a failure: {error}");
            return;
        }
    };

    log::info!("[VIRTIO] driver init was a success!!");
    config.boot();
    unsafe { log::debug!("[VIRTIO] config={:#x?}", *config.inner) };
}

fn init_pci(device: &Device, mem: &mut PciMemory) -> Result<VirtioPciCommonCfg, DriverError> {
    let mut cap = Vec::<CapData>::new();
    device.get_capabilities::<CapData, Vec<CapData>>(&mut cap);

    // cap.iter().for_each(|e| log::debug!("{e:x?}"));

    let bars: BTreeSet<u8> = cap
        .iter()
        .filter(|cap| cap.length > 0)
        .map(|cap| cap.bar)
        .collect();
    let bar_addrs = allocate_bar_addrs(bars, device, mem)?;

    let config: Option<&CapData> = cap.iter().find(|e| e.typ == CapDataType::Common);
    let data = match config {
        Some(&data) => data,
        None => unreachable!(),
    };

    // to quote osdev.wiki:
    // > Before attempting to read the information about the BAR, make sure to disable both I/O and
    // > memory decode in the command byte. /* ... */ This is needed as some devices are known to
    // > decode the write of all ones to the register as an (unintended) access.
    device.disable_io_space();
    device.disable_mem_space();

    device.enable_mem_space();

    let address = bar_addrs.get(&data.bar).ok_or(DriverError::OtherError(
        "address for bar has not been allocated",
    ))?;
    let config = unsafe { VirtioPciCommonCfg::from_raw(address + data.offset as usize) };

    Ok(config)
}

fn allocate_bar_addrs(
    bars: BTreeSet<u8>,
    device: &Device,
    mem: &mut PciMemory,
) -> Result<BTreeMap<u8, usize>, DriverError> {
    let mut bar_addrs = BTreeMap::<u8, usize>::new();
    for bar_nr in bars {
        let (is_64_bits, size) = device.get_bar_size(bar_nr);

        let address = mem.allocate(size as usize, is_64_bits);
        let address = address.ok_or(DriverError::OutOfMemoryPci)?;

        bar_addrs.insert(bar_nr, address);

        if is_64_bits {
            let addr_hi = (address >> 32) as u32;
            let addr_lo = address as u32;

            assert_eq!(address, ((addr_hi as usize) << 32) | (addr_lo as usize));

            device.write_bar(bar_nr + 1, addr_hi);
            device.write_bar(bar_nr, addr_lo);
        } else {
            log::warn!("[VIRTIO] 32-bit memory address are not tested, but the device requests it");
            device.write_bar(bar_nr, address as u32);
        }
    }

    Ok(bar_addrs)
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

struct VirtioPciCommonCfg {
    inner: *mut VirtioPciCommonCfgRaw,
}

impl VirtioPciCommonCfg {
    pub unsafe fn from_raw(addr: usize) -> Self {
        let inner = addr as *mut VirtioPciCommonCfgRaw;
        Self { inner }
    }

    pub fn boot(&self) {
        let inner = unsafe { &*self.inner };
        inner.device_feature_select.set(0xffffffff);
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
    device_status: RegCell<u8, RW>,
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
