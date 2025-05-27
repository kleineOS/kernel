//! VirtIO Block Device driver
//! current version: 0.2-dev

use alloc::vec::Vec;

use super::DriverError;
use crate::systems::pci::{Device, PciMemory};

pub const ID_PAIR: (u16, u16) = (0x1af4, 0x1001);

pub fn init(device: Device, mem: &mut PciMemory) {
    log::info!("[VIRTIO] initialising block device driver");
    match init_driver(&device, mem) {
        Ok(_) => log::info!("[VIRTIO] driver init was a success!!"),
        Err(error) => log::error!("[VIRTIO] driver init was a failure: {error}"),
    }
}

fn init_driver(device: &Device, mem: &mut PciMemory) -> Result<(), DriverError> {
    let mut cap = Vec::<CapData>::new();
    device.get_capabilities::<CapData, Vec<CapData>>(&mut cap);

    let config: Option<&CapData> = cap.iter().find(|e| e.typ == CapDataType::Common);
    let data = match config {
        Some(&data) => data,
        None => unreachable!(),
    };

    // to quote osdev.wiki:
    // > Before attempting to read the information about the BAR, make sure to disable both I/O and
    // > memory decode in the command byte. You can restore the original value after completing the
    // > BAR info read. This is needed as some devices are known to decode the write of all ones to
    // > the register as an (unintended) access.
    device.disable_io_space();
    device.disable_mem_space();

    mem.allocate_64bit(1);

    let _bar = data.bar;

    device.enable_mem_space();

    Ok(())
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
