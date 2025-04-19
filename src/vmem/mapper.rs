use bitflags::bitflags;

use crate::{PAGE_SIZE, alloc::BitMapAlloc};

const FLAG_VALID: usize = 1 << 0;
const FLAG_READ: usize = 1 << 1;
const FLAG_WRITE: usize = 1 << 2;
const FLAG_EXEC: usize = 1 << 3;
const FLAG_USER: usize = 1 << 4;

pub fn walk(balloc: &mut BitMapAlloc, base: usize, vaddr: usize, paddr: usize) -> &mut usize {
    let mut pagetable = unsafe { &mut *(base as *mut [usize; 512]) };

    for level in [2, 1] {
        let index = vaddr >> (12 + (9 * level)) & 0x1FF;
        let pte = &mut pagetable[index];

        if *pte & FLAG_VALID != 0 {
            log::info!("FIRST BRANCH");
            let pte2pa = (*pte >> 10) << 12;
            pagetable = unsafe { &mut *(pte2pa as *mut [usize; 512]) };
        } else {
            log::info!("SECOND BRANCH");
            let new_page = balloc.alloc(1);
            unsafe { core::ptr::write_bytes(new_page as *mut u8, 0, PAGE_SIZE) };
            pagetable = unsafe { &mut *(new_page as *mut [usize; 512]) };

            let paddr_for_pte = paddr >> 12 << 10;
            *pte = paddr_for_pte | FLAG_VALID;
        }
    }

    let index = vaddr >> 12 & 0x1FF;
    &mut pagetable[index]
}

pub fn map(
    balloc: &mut BitMapAlloc,
    base: usize,
    vaddr: usize,
    paddr: usize,
    perms: Perms,
    size: usize,
) {
    assert!(vaddr % PAGE_SIZE == 0);
    assert!(size % PAGE_SIZE == 0);
    assert!(size > 0);

    let mut va = vaddr;
    let last = vaddr + size - PAGE_SIZE;
    let mut pa = paddr;

    loop {
        let pte = walk(balloc, base, va, pa);

        if *pte & FLAG_VALID != 0 {
            panic!("remap");
        }

        let paddr_for_pte = paddr >> 12 << 10;
        *pte = paddr_for_pte | perms.bits() | FLAG_VALID;

        assert!(va <= last);
        if va == last {
            break;
        }

        va += PAGE_SIZE;
        pa += PAGE_SIZE;
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
