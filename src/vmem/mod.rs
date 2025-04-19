#![allow(unused)]

mod mapper;

use core::sync::atomic::{AtomicUsize, Ordering};

use packed_struct::derive::PackedStruct;
use spin::Mutex;

use crate::{PAGE_SIZE, alloc::BitMapAlloc, riscv};
use mapper::*;

const UNINITALISED: usize = 0xdeadbabe;
const MODE_SV39: usize = 8usize << 60;

static PAGE_TABLE: AtomicUsize = AtomicUsize::new(UNINITALISED);

pub fn init(balloc: &mut BitMapAlloc) {
    if PAGE_TABLE.load(Ordering::Relaxed) != UNINITALISED {
        panic!("vmem::init called twice");
    }

    let page_table = balloc.alloc(1);
    unsafe { core::ptr::write_bytes(page_table as *mut u8, 0, crate::PAGE_SIZE) };

    map(
        balloc,
        page_table,
        0x80200000,
        0x80200000,
        Perms::all(),
        0x4000,
    );

    // map(
    //     &balloc,
    //     page_table,
    //     0x80200000,
    //     0x80200000,
    //     Perms::all(),
    //     4096 * 40,
    // );

    PAGE_TABLE.store(page_table, Ordering::Relaxed);
}

pub fn inithart() {
    if PAGE_TABLE.load(Ordering::Relaxed) == UNINITALISED {
        panic!("call vmem::inithart called before calling vmem::init");
    }

    riscv::sfence_vma();

    let kptbl = PAGE_TABLE.load(Ordering::Relaxed);

    let satp_entry = MODE_SV39 | (kptbl >> 12);

    riscv::satp::write(satp_entry);

    log::info!("satp entry: {:#x}", satp_entry);

    riscv::sfence_vma();
}
