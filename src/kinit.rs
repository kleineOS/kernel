//! Second stage of the kernel's init

use crate::allocator::BitMapAlloc;
use crate::riscv::{self, sbi};
use crate::{PAGE_SIZE, STACK_PAGES, vmem};

const STACK_SIZE: usize = STACK_PAGES * PAGE_SIZE;

/// 1. Allocate stacks for all available harts
pub fn pre_kinit(balloc: &mut BitMapAlloc, fdt: fdt::Fdt) {
    let cpu_count = fdt.cpus().count();

    for id in 0..cpu_count {
        let addr = balloc.alloc(STACK_PAGES);
        // the address we are returned is at the top of the allocated space, we need to go lower
        let stack_bottom = addr + STACK_SIZE;
        sbi::hsm::start(id, _start as usize, stack_bottom);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kinit(hartid: usize) -> ! {
    // safety: cannot be used in critical section
    unsafe { riscv::interrupt::enable_all() };
    crate::trap::reset_timer();

    vmem::inithart();

    log::info!("[HART#{hartid}] Entering loop...");
    riscv::pauseloop();
}

unsafe extern "C" {
    fn _start();
}
