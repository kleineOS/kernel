#![allow(unused)]

#[repr(C)]
pub struct VirtQDesc {
    addr: u64,
    len: u32,
    flags: VirtQDescFlags,
    next: u16, // next field if flags contain NEXT
}

#[repr(C)]
pub struct VirtQAvail<const N: usize> {
    flags: VirtQAvailFlags,
    idx: u16,
    ring: [u16; N],
    used_event: u16,
}

#[repr(C)]
pub struct VirtQUsed<const N: usize> {
    flags: VirtQUsedFlags,
    idx: u16,
    ring: [VirtQUsedElem; N],
    avail_event: u16,
}

#[repr(C)]
pub struct VirtQUsedElem {
    id: u32,
    len: u32,
}

bitflags::bitflags! {
    pub struct VirtQDescFlags: u16 {
        /// This marks a buffer as continuing via the next field
        const NEXT = 1;
        /// This marks a buffer as device write-only (otherwise device read-only)
        const WRITE = 2;
        /// This means the buffer contains a list of buffer descriptors
        const INDIRECT = 4;
    }

    pub struct VirtQAvailFlags: u16 {
        const NO_INTERRUPT = 1;
    }

    pub struct VirtQUsedFlags: u16 {
        const NO_NOTIFY = 1;
    }
}
