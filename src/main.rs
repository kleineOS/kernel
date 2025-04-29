#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

mod allocator;
mod drivers;
mod proc;
mod riscv;
mod trap;
mod vmem;
mod writer;

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};

use linked_list_allocator::LockedHeap;
use riscv::sbi;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

unsafe extern "C" {
    pub static ETEXT: usize;
    pub static STACK_TOP: usize;
    pub static STACK_BOTTOM: usize;
    // reserved for a "dma" stype allocator (contigous allocations)
    pub static HEAP0_TOP: usize;
    // reserved for a global_alloc which enables me to use `alloc`
    pub static HEAP1_TOP: usize;
}

pub const INTERVAL: usize = 8000000;
pub const PAGE_SIZE: usize = 0x1000; // 4096

fn is_main_hart() -> bool {
    static INIT_DONE: AtomicBool = AtomicBool::new(false);
    INIT_DONE
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_ok()
}

fn init_heap1() {
    let heap_start = unsafe { HEAP1_TOP as *mut u8 };
    let size = 256 * 1024 * 1024; // 256M

    unsafe { ALLOCATOR.lock().init(heap_start, size / 8) }
}

#[unsafe(no_mangle)]
extern "C" fn start(hartid: usize, fdt_ptr: usize) -> ! {
    println!("\n\n\n^w^ welcome to my operating system");

    assert_eq!(size_of::<usize>(), 64 / 8, "we only support 64-bit");
    if !is_main_hart() {
        todo!("multi threading");
    }

    writer::init_log();
    log::debug!("HART#{hartid}");

    let balloc_addr = unsafe { HEAP0_TOP };
    let balloc = allocator::BitMapAlloc::init(balloc_addr);
    init_heap1();

    // safety: the fdt_ptr needs to be valid. this is "guaranteed" by OpenSBI
    let _fdt = unsafe { fdt::Fdt::from_ptr(fdt_ptr as *const u8) }.expect("could not parse fdt");

    {
        let mut balloc = balloc.lock();
        let mut mapper = vmem::init(&mut balloc);

        let heap1 = unsafe { HEAP1_TOP };
        let size = 256 * 1024 * 1024;
        let pages = (size / PAGE_SIZE) + 1;
        mapper.map(heap1, heap1, vmem::Perms::READ_WRITE, pages);

        // uart::UartDriver::init(fdt, &mut mapper).expect("could not init uart driver");
    }

    #[cfg(test)]
    test_main();

    kmain();
}

fn kmain() -> ! {
    // safety: cannot be used in critical section
    unsafe { riscv::interrupt::enable_all() };
    sbi::time::set_timer(riscv::time() + INTERVAL);

    vmem::inithart();

    for i in alloc::vec![1, 2, 3, 5] {
        log::info!("reading from a dynamic vec {i}");
    }

    log::info!("Entering loop...");
    riscv::pauseloop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    riscv::interrupt::disable();

    #[cfg(test)]
    println!("[TEST FAILED]");
    println!("{}", info);

    riscv::pauseloop();
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) -> ! {
    use sbi::srst::*;

    println!("\n\n");
    println!("Running tests...");

    for (i, test) in tests.iter().enumerate() {
        println!("\nrunning test #{i}");
        test();
        println!("test #{i} [OK]");
    }

    system_reset(ResetType::Shutdown);
    riscv::pauseloop();
}

// ========= ASSEMBLY IMPORTS =========
include_asm!("kernelvec.s");
include_asm!("entry.s");
// ====================================
