#![allow(unused)]

use core::sync::atomic::{AtomicUsize, Ordering};

use bitflags::bitflags;
use spin::Mutex;

use crate::{alloc::BitMapAlloc, riscv};

const UNINITALISED: usize = 0xdeadbabe;
const MODE_SV39: usize = 8usize << 60;

static PAGE_TABLE: AtomicUsize = AtomicUsize::new(UNINITALISED);

bitflags! {
    pub struct Perms: usize {
        const READ = 1 << 1;
        // const WRITE = 1 << 2; // WRITE cannot be set without READ
        const READ_WRITE = 1 << 2 | 1 << 1;
        const EXEC = 1 << 3;
        const USER = 1 << 4;
    }
}

fn map_page(base: usize, vaddr: usize, phyaddr: usize, perms: Perms) {
    log::debug!(
        "mapping {:#x} to {:#x} with perms: {:#b}",
        phyaddr,
        vaddr,
        perms
    );
}

pub fn init(balloc: Mutex<BitMapAlloc>) {
    let mut balloc = balloc.lock();
    let page_table = balloc.alloc(1);

    unsafe { core::ptr::write_bytes(page_table as *mut u8, 0, crate::PAGE_SIZE) };

    map_page(page_table, 0x80200000, 0x80200000, Perms::EXEC);

    PAGE_TABLE.store(page_table, Ordering::Relaxed);
}

pub fn inithart() {
    riscv::sfence_vma();

    let kptbl = PAGE_TABLE.load(Ordering::Relaxed);

    let satp_entry = MODE_SV39 | (kptbl >> 12);

    riscv::satp::write(satp_entry);

    log::info!("satp entry: {:#x}", satp_entry);

    riscv::sfence_vma();
}
