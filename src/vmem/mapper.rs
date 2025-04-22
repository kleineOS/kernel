#![allow(unused)]

use bitflags::bitflags;

use crate::alloc::BitMapAlloc;

fn walk(balloc: &mut BitMapAlloc, root: *mut [usize; 512], vaddr: usize) -> *mut usize {
    let mut pagetable = root;

    for level in [2, 1] {
        let pte = unsafe { &mut (*pagetable)[px(level, vaddr)] };

        if *pte & PTE_V != 0 {
            // PTE is valid, get the next level page table
            pagetable = unsafe { (pte2pa(*pte) as *mut [usize; 512]) };
        } else {
            // PTE is not valid, allocate a new page table
            let new_page = balloc.alloc(1) as *mut [usize; 512];

            // Zero out the new page table
            unsafe {
                (*new_page).fill(0);
            }

            // Set the PTE to point to the new page table
            *pte = pa2pte(new_page as usize) | PTE_V;

            pagetable = new_page;
        }
    }

    unsafe { &mut (*pagetable)[px(0, vaddr)] }
}

pub fn map(
    balloc: &mut BitMapAlloc,
    root: *mut [usize; 512],
    vaddr: usize,
    paddr: usize,
    perms: Perms,
    pages: usize,
) {
    for i in 0..pages {
        let offset = 4096 * i;
        let va = vaddr + offset;
        let pa = paddr + offset;

        let pte = walk(balloc, root, va);

        unsafe {
            if *pte & PTE_V != 0 {
                panic!("idfk man, just panic");
            }

            *pte = pa2pte(pa) | perms.bits() | PTE_V;
        }
    }
}

bitflags! {
    pub struct Perms: usize {
        const READ = 1 << 1;
        const READ_WRITE = 1 << 1 | 1 << 2;
        const EXEC = 1 << 3;
    }
}

const PTE_V: usize = 1 << 0;

#[inline]
fn px(level: usize, va: usize) -> usize {
    (va >> (12 + (9 * level))) & 0x1FF
}

#[inline]
fn pte2pa(pte: usize) -> usize {
    (pte >> 10) << 12
}

#[inline]
fn pa2pte(pa: usize) -> usize {
    (pa >> 12) << 10
}
