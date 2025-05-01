//! Second stage of the kernel's init

use crate::riscv::{self, sbi};
use crate::{INTERVAL, vmem};

pub fn kinit(hartid: usize, _fdt: fdt::Fdt) -> ! {
    // safety: cannot be used in critical section
    unsafe { riscv::interrupt::enable_all() };

    sbi::time::set_timer(riscv::time() + INTERVAL);
    vmem::inithart();

    log::info!("[HART#{hartid}] Entering loop...");
    riscv::pauseloop();
}
