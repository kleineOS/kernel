crate::include_asm!("symbols.s");

unsafe extern "C" {
    pub static MEMTOP: usize;
    pub static ETEXT: usize;

    // pub static STACK_TOP: usize;
    // pub static STACK_BOTTOM: usize;

    // reserved for a "dma" stype allocator (contigous allocations)
    pub static HEAP0_TOP: usize;
    // reserved for a global_alloc which enables me to use `alloc`
    pub static HEAP1_TOP: usize;
}
