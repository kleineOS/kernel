//! Second stage of the kernel's init

use crate::riscv::{self, sbi};
use crate::{INTERVAL, vmem};

#[unsafe(no_mangle)]
pub extern "C" fn kinit(hartid: usize) -> ! {
    // safety: cannot be used in critical section
    unsafe { riscv::interrupt::enable_all() };

    sbi::time::set_timer(riscv::time() + INTERVAL);
    vmem::inithart();

    log::info!("[HART#{hartid}] Entering loop...");
    riscv::pauseloop();
}
