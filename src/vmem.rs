use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{alloc::BitMapAlloc, riscv};

static PAGE_TABLE: AtomicUsize = AtomicUsize::new(0xdead_babe);

pub fn init(balloc: &mut BitMapAlloc) {
    let tbl_addr = balloc.alloc(1);
    PAGE_TABLE.store(tbl_addr, Ordering::Relaxed);

    let table = unsafe { &mut *(tbl_addr as *mut [usize; 512]) };
    log::info!("root page table is at {:#x?}", tbl_addr);
    map(balloc, table, 0x8020_0000, 0x8020_0000, 0xe, 4);
}

fn walk(balloc: &mut BitMapAlloc, mut pagetable: &mut [usize; 512], vaddr: usize) -> *mut usize {
    for level in [2, 1].iter() {
        let idx = PX(*level, vaddr);
        let pte = &mut pagetable[idx];

        if (*pte & (1 << 0)) != 0 {
            let pa = PTE2PA(*pte);
            pagetable = unsafe { &mut *(pa as *mut [usize; 512]) };
        } else {
            let new_table_addr = balloc.alloc(1);
            let new_table = unsafe { &mut *(new_table_addr as *mut [usize; 512]) };

            for entry in new_table.iter_mut() {
                *entry = 0;
            }

            *pte = PA2PTE(new_table_addr) | (1 << 0); // Set valid bit

            pagetable = new_table;
        }
    }

    &mut pagetable[PX(0, vaddr)]
}

fn map(
    balloc: &mut BitMapAlloc,
    root: &mut [usize; 512],
    paddr: usize,
    vaddr: usize,
    perms: usize,
    pages: usize,
) {
    assert!(vaddr & 4096 == 0);
    assert!(pages > 0);

    for i in 0..pages {
        let offset = 4096 * i;
        let va = vaddr + offset;
        let pa = paddr + offset;

        let pte_ptr = walk(balloc, root, va);

        unsafe {
            if *pte_ptr & 1 << 0 != 0 {
                panic!("remap");
            }

            *pte_ptr = PA2PTE(pa) | perms | 1 << 0;
        }
    }
}

pub fn inithart() {
    let kptbl = PAGE_TABLE.load(Ordering::Relaxed);

    const MODE_SV39: usize = 8usize << 60;
    let satp_entry = MODE_SV39 | (kptbl >> 12);

    unsafe {
        let pc: usize;
        core::arch::asm!("auipc {}, 0", out(reg) pc);
        log::info!("Current PC: {:#x}", pc);
    }

    riscv::sfence_vma();
    riscv::satp::write(satp_entry);
    riscv::sfence_vma();

    log::info!("satp set to value: {:#x}", satp_entry);
}

#[inline]
#[allow(non_snake_case)]
pub fn PX(level: usize, va: usize) -> usize {
    (va >> (12 + (9 * level))) & 0x1FF
}

#[inline]
#[allow(non_snake_case)]
pub fn PTE2PA(pte: usize) -> usize {
    (pte >> 10) << 12
}

#[inline]
#[allow(non_snake_case)]
pub fn PA2PTE(pa: usize) -> usize {
    (pa >> 12) << 10
}
