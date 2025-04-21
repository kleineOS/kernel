use bitflags::bitflags;

use crate::alloc::BitMapAlloc;

pub fn map(
    balloc: &mut BitMapAlloc,
    root: &mut [PageTableEntry; 512],
    vaddr: usize,
    paddr: usize,
    perms: Perms,
) {
    let level = 0;

    let vpn = [
        (vaddr >> 12) & VPN_MASK,
        (vaddr >> 21) & VPN_MASK,
        (vaddr >> 30) & VPN_MASK,
    ];

    let ppn = [
        (paddr >> 12) & PPN_MASK,
        (paddr >> 21) & PPN_MASK,
        (paddr >> 30) & PPN_MASK_BIG,
    ];

    let mut v = &mut root[vpn[2]];

    for i in (level..2).rev() {
        if !v.is_valid() {
            let page = balloc.alloc(1);
            unsafe { core::ptr::write_bytes(page as *mut u8, 0, 4096) };
            v.set_inner(page >> 2);
            v.set_valid(true);
        }

        let entry = ((v.get_inner() & !0x3ff) << 2) as *mut PageTableEntry;
        v = unsafe { entry.add(vpn[i]).as_mut().unwrap() };
    }

    let entry = (ppn[2] << 28) | (ppn[1] << 19) | (ppn[0] << 10) | perms.bits();
    v.set_inner(entry);
    v.set_valid(true);
}

#[repr(C)]
pub struct PageTableEntry {
    inner: usize,
}

impl PageTableEntry {
    pub fn set_inner(&mut self, value: usize) {
        self.inner = value
    }

    pub fn get_inner(&self) -> usize {
        self.inner
    }

    pub fn is_valid(&self) -> bool {
        (FLAG_VALID & self.inner) != 0
    }

    pub fn set_valid(&mut self, value: bool) {
        self.inner &= !FLAG_VALID;
        self.inner |= if value { FLAG_VALID } else { 0 };
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct Perms: usize {
        const READ = FLAG_READ;
        // WRITE cannot be set without READ
        const READ_WRITE = FLAG_WRITE | FLAG_READ;
        const EXEC = FLAG_EXEC;
        const USER = FLAG_USER;
    }
}

const PPN_MASK_BIG: usize = 0x3ff_ffff; // 26 bits
const PPN_MASK: usize = 0x1ff; // 9 bits
const VPN_MASK: usize = 0x1ff; // 9 bits

const FLAG_VALID: usize = 1 << 0;
const FLAG_READ: usize = 1 << 1;
const FLAG_WRITE: usize = 1 << 2;
const FLAG_EXEC: usize = 1 << 3;
const FLAG_USER: usize = 1 << 4;
