//! VirtIO Block Device driver
//! current version: 0.2-dev

use alloc::vec::Vec;

use super::DriverError;
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
    log::debug!("[VIRTIO] config={config:#x?}");
}

fn init_pci(
    device: &Device,
    mem: &mut PciMemory,
) -> Result<&'static mut VirtioPciCommonCfg, DriverError> {
    let mut cap = Vec::<CapData>::new();
    device.get_capabilities::<CapData, Vec<CapData>>(&mut cap);

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

    let bar_nr = data.bar;
    let (is_64_bits, size) = device.get_bar_size(bar_nr);

    let address = mem.allocate(size as usize, is_64_bits);
    let address = address.ok_or(DriverError::OutOfMemoryPci)?;

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

    device.enable_mem_space();

    let config = (address + data.offset as usize) as *mut VirtioPciCommonCfg;
    let config = unsafe { &mut (*config) };

    Ok(config)
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
