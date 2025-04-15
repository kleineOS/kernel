#![no_std]
#![no_main]
#![feature(custom_test_frameworks, abi_riscv_interrupt)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

mod alloc;
mod riscv;
mod trap;
mod vmem;
mod writer;

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};

use riscv::sbi;

unsafe extern "C" {
    pub static STACK_TOP: usize;
    pub static STACK_BOTTOM: usize;
    pub static HEAP_TOP: usize;
}

pub const INTERVAL: usize = 8000000;
pub const UART0: usize = 0x10000000;

fn is_main_hart() -> bool {
    // false if the global init has not yet been done
    static INIT_DONE: AtomicBool = AtomicBool::new(false);

    // weird function, here is tldr:
    // the function returns Some(_) if the expected state is `false` (first arg)
    // if so, then the expected state is atomically modified to `true` (second arg)
    // the third and fourth arguments define the ordering for atomic operations
    INIT_DONE
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_ok()
}

#[unsafe(no_mangle)]
extern "C" fn start(hartid: usize, fdt_ptr: usize) -> ! {
    println!("\n\n\n^w^ welcome to my operating system");

    assert_eq!(size_of::<usize>(), 64 / 8, "we only support 64-bit");
    // not a requirement, but we define our linker script this way and it is easy to define rules
    // in asserts so we know if we messed up somewhere when modifying the linker script
    unsafe { assert_eq!(STACK_BOTTOM, HEAP_TOP, "heap must come after the stack") };

    if !is_main_hart() {
        todo!("multi threading");
    }

    writer::init_log();
    log::debug!("HART#{hartid}");

    // safety: the fdt_ptr needs to be valid. this is "guaranteed" by OpenSBI
    let fdt = unsafe { fdt::Fdt::from_ptr(fdt_ptr as *const u8) }.expect("could not parse fdt");

    let cpu_count = fdt.cpus().count();
    log::info!("total {} harts", cpu_count);

    unsafe {
        core::arch::asm!(
            "li a0, 0xdeadbeef
            unimp"
        )
    };

    let table = alloc::init();

    for _ in 0..4095 {
        let _ptr = alloc::alloc1(table);
    }

    kmain();
}

fn kmain() -> ! {
    // safety: cannot be used in critical section
    unsafe { riscv::interrupt::enable_all() };
    sbi::time::set_timer(riscv::time() + INTERVAL);

    log::info!("Entering loop...");
    riscv::pauseloop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    riscv::interrupt::disable();
    println!("{}", info);
    loop {}
}

#[cfg(test)]
pub fn test_runner(_: &[&dyn Fn()]) {
    panic!("tests not implimented");
}

// ========= ASSEMBLY IMPORTS =========
include_asm!("kernelvec.s");
include_asm!("entry.s");
// ====================================
