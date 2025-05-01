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

mod docs {
    //! Documentation
    //!
    //! The kernel initialisation is split into several parts:
    //! 1. entry.S: This sets up a stack, sets a trapvec and calls the next stage
    //! 2. start: This does a lot of the basic init, but it only runs on the main hart. One of my
    //!    goals with this step is to make sure everything is allocated on the heap, essentially
    //!    storing no state to the thread-local stack.
    //!     - Sets up our writer and the log crate to use OpenSBI DBCN extension for logging
    //!     - Parse the Flattened Device Tree into a [fdt::Fdt] struct
    //!     - Initialise our BitMap allocator for allocating contigous pages
    //!     - Setup an external linked list allocator as our global_allocator (for the alloc crate)
    //!     - Initialise the mapper for mapping vmem (WE DO NOT ENABLE VMEM YET)
    //!     - Initialise a Driver Manager (todo: we will manually initialise drivers for now)
    //!     - Set ra for self and all harts to the next stage and boot up all harts, then jump to ra
    //! 3. kinit (this file): This is the final stage of what can be considered "init". This stage runs
    //!    the same for all harts, the concept of a "main hart" dissolves before switching to this
    //!     - Enable all interrupts, a small timer for scheduling and enable vmem
    //!     - Add the userspace `init` process to a process queue
    //!     - TODO: idk how to do scheduling or processes, so we are here for now
}
