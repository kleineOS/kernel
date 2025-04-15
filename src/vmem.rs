#![allow(unused)]

use bitflags::bitflags;

pub fn init() {
    let a = PageTableEntry::VALID | PageTableEntry::READ;
}

bitflags! {
    #[derive(Debug)]
    struct PageTableEntry: u64 {
        // these bits the actual "properties" of the pte
        const VALID     = 1 << 0;
        const READ      = 1 << 1;
        const WRITE     = 1 << 2;
        const EXECUTE   = 1 << 3;
        const GLOBAL    = 1 << 5;
        const ACCESSED  = 1 << 6;
        const DIRTY     = 1 << 7;
        // bits 8-9 are reserved

        // 0x1FF == 9 bits
        const PPN0      = 0x1FF << 10;
        const PPN1      = 0x1FF << 19;
        // 0x3FFFFFF == 26 bits
        const PPN2      = 0x3FFFFFF << 28;

        // bits 54-60 are reserved
        // bits 61-62 are for Svpbmt (we dont use)
        // bit 63 is for Svnapot (we dont use)
    }
}

const VMEM_MODE: Mode = Mode::Sv39;
enum Mode {
    Sv39,
}
