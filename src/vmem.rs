#![allow(unused)]

use core::sync::atomic::{AtomicUsize, Ordering};

use bitflags::bitflags;
use packed_struct::derive::PackedStruct;
use spin::Mutex;

use crate::{PAGE_SIZE, alloc::BitMapAlloc, riscv};

const UNINITALISED: usize = 0xdeadbabe;
const MODE_SV39: usize = 8usize << 60;

static PAGE_TABLE: AtomicUsize = AtomicUsize::new(UNINITALISED);

const FLAG_VALID: usize = 1 << 0;
const FLAG_READ: usize = 1 << 1;
const FLAG_WRITE: usize = 1 << 2;
const FLAG_EXEC: usize = 1 << 3;
const FLAG_USER: usize = 1 << 4;

struct PageTableEntry {
    valid: bool,
    perms: Perms,
    paddr: usize,
}

impl PageTableEntry {
    fn read_from_usize(value: usize) {
        let valid = value & FLAG_VALID != 0;
        // bits 1 = read, 2 = write, 3 = exec, 4 = user
        let perms = Perms::from_bits_truncate(value & !FLAG_VALID);
        log::debug!("valid: {valid}, perms: {perms:#b}");
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

const PAGING_LEVELS: usize = 3;

pub fn init(balloc: Mutex<BitMapAlloc>) {
    if PAGE_TABLE.load(Ordering::Relaxed) != UNINITALISED {
        panic!("vmem::init called twice");
    }

    let mut balloc0 = balloc.lock();
    let page_table = balloc0.alloc(1);
    drop(balloc0);

    unsafe { core::ptr::write_bytes(page_table as *mut u8, 0, crate::PAGE_SIZE) };

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
