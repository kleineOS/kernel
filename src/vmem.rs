//! TODO:
//! - [ ] Use AtomicPtr instead of AtomicUsize
//! - [ ] Replace the .expect and panic! with Result
//! - [ ] Dynamically read the start of the .text section

use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{alloc::BitMapAlloc, riscv};

static PAGE_TABLE: AtomicUsize = AtomicUsize::new(0xdead_babe);

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct Perms: usize {
        const READ = 1 << 1;
        // WRITE without READ is an invalid state
        const READ_WRITE = Self::READ.bits() | 1 << 2 ;
        const EXEC = 1 << 3;
    }
}

/// A struct that holds a reference to an allocator and the root page table. This allows drivers to
/// map pages, without having to worry about anything outside their scope
pub struct Mapper<'a> {
    balloc: &'a mut BitMapAlloc,
    table: &'a mut [PTEntry; 512],
}

impl<'a> Mapper<'a> {
    pub fn map(&mut self, paddr: usize, vaddr: usize, perms: Perms, pages: usize) {
        map(self.balloc, self.table, paddr, vaddr, perms, pages);
    }
}

#[repr(C)]
struct PTEntry {
    inner: usize,
}

impl PTEntry {
    fn is_valid(&self) -> bool {
        (self.inner & (1 << 0)) != 0
    }

    fn set_valid(&mut self, valid: bool) {
        let valid = if valid { 1 << 0 } else { 0 };
        self.inner |= valid;
    }

    fn set_perms(&mut self, perms: Perms) {
        self.inner |= perms.bits();
    }

    /// set inner from physical address
    fn set_inner_from_pa(&mut self, paddr: usize) {
        self.inner = (paddr >> 12) << 10;
    }

    fn get_physical_addr(&self) -> usize {
        (self.inner >> 10) << 12
    }

    fn clear(&mut self) {
        self.inner = 0
    }
}

const KERNEL_START: usize = 0x8020_0000;

pub fn init(balloc: &mut BitMapAlloc) -> Mapper {
    let etext = unsafe { crate::ETEXT };
    // round it up to 4096 bytes
    let etext = (etext + 4095) & !4095;
    let kernel_pages = (etext - KERNEL_START) / 4096;

    let tbl_addr = balloc.alloc(1);
    PAGE_TABLE.store(tbl_addr, Ordering::Relaxed);

    // safety: we know that table_addr contains 4096 bytes, so this is safe
    let table = unsafe { &mut *(tbl_addr as *mut [PTEntry; 512]) };

    map(
        balloc,
        table,
        KERNEL_START,
        KERNEL_START,
        Perms::EXEC,
        kernel_pages,
    );
    map(balloc, table, etext, etext, Perms::READ_WRITE, 200);

    Mapper { balloc, table }
}

fn map(
    balloc: &mut BitMapAlloc,
    root: &mut [PTEntry; 512],
    paddr: usize,
    vaddr: usize,
    perms: Perms,
    pages: usize,
) {
    assert!(vaddr & 4096 == 0);
    assert!(pages > 0);

    for i in 0..pages {
        let offset = 4096 * i;
        let va = vaddr + offset;
        let pa = paddr + offset;

        let pte = unsafe {
            walk(balloc, root, va)
                .as_mut()
                .expect("could not dereference pointer")
        };

        // the walk function does not modify the page table entries on level 0. So if the valid bit
        // is flipped, the user has most likely mapped an overlapping region
        if pte.is_valid() {
            panic!("remap detected, aborting");
        }

        pte.set_inner_from_pa(pa);
        pte.set_perms(perms);
        pte.set_valid(true);
    }
}

fn walk(
    balloc: &mut BitMapAlloc,
    mut pagetable: &mut [PTEntry; 512],
    vaddr: usize,
) -> *mut PTEntry {
    for level in [2, 1].iter() {
        let idx = idx_for_vaddr(*level, vaddr);
        let pte = &mut pagetable[idx];

        if pte.is_valid() {
            let pa = pte.get_physical_addr();
            pagetable = unsafe { &mut *(pa as *mut [PTEntry; 512]) };
        } else {
            let new_table_addr = balloc.alloc(1);
            let new_table = unsafe { &mut *(new_table_addr as *mut [PTEntry; 512]) };

            for entry in new_table.iter_mut() {
                entry.clear()
            }

            pte.set_inner_from_pa(new_table_addr);
            pte.set_valid(true);

            pagetable = new_table;
        }
    }

    &mut pagetable[idx_for_vaddr(0, vaddr)]
}

pub fn inithart() {
    let kptbl = PAGE_TABLE.load(Ordering::Relaxed);

    const MODE_SV39: usize = 8usize << 60;
    let satp_entry = MODE_SV39 | (kptbl >> 12);

    riscv::sfence_vma();
    riscv::satp::write(satp_entry);
    riscv::sfence_vma();

    log::debug!("satp set to value: {:#x}", satp_entry);
}

#[inline]
pub fn idx_for_vaddr(level: usize, va: usize) -> usize {
    (va >> (12 + (9 * level))) & 0x1FF
}
