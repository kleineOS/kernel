mod mapper;

use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{PAGE_SIZE, alloc::BitMapAlloc, riscv};
use mapper::*;

const UNINITALISED: usize = 0xdeadbabe;
const MODE_SV39: usize = 8usize << 60;

static PAGE_TABLE: AtomicUsize = AtomicUsize::new(UNINITALISED);

pub fn init(balloc: &mut BitMapAlloc) {
    if PAGE_TABLE.load(Ordering::Relaxed) != UNINITALISED {
        panic!("vmem::init called twice");
    }

    let page_table_addr = balloc.alloc(1);
    unsafe { core::ptr::write_bytes(page_table_addr as *mut u8, 0, PAGE_SIZE) };
    log::info!("page table root at: {:#x}", page_table_addr);

    PAGE_TABLE.store(page_table_addr, Ordering::Relaxed);

    let page_table = page_table_addr as *mut [usize; 512];

    map(
        balloc,
        page_table,
        0x8000_0000,
        0x8000_0000,
        Perms::all(),
        20,
    );
}

#[unsafe(no_mangle)]
pub fn inithart() {
    let kptbl = PAGE_TABLE.load(Ordering::Relaxed);
    assert_ne!(kptbl, UNINITALISED);

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
