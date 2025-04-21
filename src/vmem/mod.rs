mod mapper;

use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{PAGE_SIZE, alloc::BitMapAlloc, riscv};
use mapper::{PageTableEntry, *};

const UNINITALISED: usize = 0xdeadbabe;
const MODE_SV39: usize = 8usize << 60;

static PAGE_TABLE: AtomicUsize = AtomicUsize::new(UNINITALISED);

pub fn init(balloc: &mut BitMapAlloc) {
    if PAGE_TABLE.load(Ordering::Relaxed) != UNINITALISED {
        panic!("vmem::init called twice");
    }

    let page_table_addr = balloc.alloc(1);
    unsafe { core::ptr::write_bytes(page_table_addr as *mut u8, 0, PAGE_SIZE) };

    PAGE_TABLE.store(page_table_addr, Ordering::Relaxed);

    let page_table = unsafe { &mut *(page_table_addr as *mut [PageTableEntry; 512]) };

    for i in 0..10000 {
        let offset = i * 4096;
        map(
            balloc,
            page_table,
            0x8000_0000 + offset,
            0x8000_0000 + offset,
            Perms::all(),
        );
    }
}

#[unsafe(no_mangle)]
pub fn inithart() {
    if PAGE_TABLE.load(Ordering::Relaxed) == UNINITALISED {
        panic!("call vmem::inithart called before calling vmem::init");
    }

    riscv::sfence_vma();

    let kptbl = PAGE_TABLE.load(Ordering::Relaxed);

    let satp_entry = MODE_SV39 | (kptbl >> 12);
    log::info!("PRE: satp set to value: {:#x}", satp_entry);

    riscv::satp::write(satp_entry);
    riscv::sfence_vma();
}
