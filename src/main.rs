#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

mod allocator;
mod drivers;
mod kinit;
mod proc;
mod riscv;
mod trap;
mod vmem;
mod writer;

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};

use drivers::uart::CharDriver;
use linked_list_allocator::LockedHeap;
use vmem::{Mapper, Perms};

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

const KERNEL_START: usize = 0x8020_0000;
pub const INTERVAL: usize = 8000000;
pub const PAGE_SIZE: usize = 0x1000; // 4096
pub const HEAP1_SIZE: usize = 1024 * 1024 * 1024;

fn is_main_hart() -> bool {
    static INIT_DONE: AtomicBool = AtomicBool::new(false);
    INIT_DONE
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_ok()
}

fn init_heap1() {
    let heap_start = unsafe { HEAP1_TOP as *mut u8 };
    unsafe { ALLOCATOR.lock().init(heap_start, HEAP1_SIZE) }
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
    let fdt = unsafe { fdt::Fdt::from_ptr(fdt_ptr as *const u8) }.expect("could not parse fdt");

    let mut balloc = balloc.lock();
    let mut mapper = vmem::init(&mut balloc);

    // map the kernel, stack and the heap onto the memory
    map_vitals(&mut mapper).expect("could not map vital memory");

    CharDriver::init(fdt, &mut mapper).expect("could not init uart driver");
    CharDriver::log_addr().unwrap(); // cannot fail

    #[cfg(test)]
    test_main();

    // TODO: figure out how to reset the call stack and jump to this directly
    kinit::kinit(hartid, fdt);
}

fn map_vitals(mapper: &mut Mapper) -> Result<(), vmem::MapError> {
    let etext = (unsafe { ETEXT } + 4095) & !4095;
    let kernel_pages = (etext - KERNEL_START) / 4096;

    let heap1 = unsafe { HEAP1_TOP };
    let heap1_pages = HEAP1_SIZE / PAGE_SIZE;

    let stack_heap0_size = heap1 - etext;
    let stack_heap0_pages = stack_heap0_size / PAGE_SIZE;

    // MAP THE KERNEL
    mapper.map(KERNEL_START, KERNEL_START, Perms::EXEC, kernel_pages)?;
    // MAP STACK AND HEAP0
    mapper.map(etext, etext, Perms::READ_WRITE, stack_heap0_pages)?;
    // MAP HEAP1
    mapper.map(heap1, heap1, Perms::READ_WRITE, heap1_pages)?;

    Ok(())
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    riscv::interrupt::disable();

    #[cfg(test)]
    println!("[TEST FAILED]");
    println!("{}", info);

    #[cfg(test)]
    {
        use riscv::sbi::srst::*;
        system_reset(ResetType::Shutdown, ResetReason::Failure);
    }

    riscv::pauseloop();
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) -> ! {
    use riscv::sbi::srst::*;

    println!("\n\n");
    println!("Running tests...");

    for (i, test) in tests.iter().enumerate() {
        println!("\nrunning test #{i}");
        test();
        println!("test #{i} [OK]");
    }

    println!("\n=====| ALL TESTS PASSED |=====");
    system_reset(ResetType::Shutdown, ResetReason::None);
    riscv::pauseloop();
}

// ========= ASSEMBLY IMPORTS =========
include_asm!("kernelvec.s");
include_asm!("entry.s");
// ====================================
