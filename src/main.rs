#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

mod allocator;
mod drivers;
mod kinit;
mod pci;
mod proc;
mod riscv;
mod symbols;
mod systems;
mod trap;
mod vmem;
mod writer;

use core::panic::PanicInfo;

use drivers::{uart::CharDriver, virtio};
use linked_list_allocator::LockedHeap;
use pci::PcieManager;
use systems::pci::PciSubsystem;
use vmem::{Mapper, Perms};

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub const INTERVAL: usize = 8000000;
pub const PAGE_SIZE: usize = 0x1000; // 4096
pub const HEAP1_SIZE: usize = 1024 * 1024 * 1024;
pub const STACK_PAGES: usize = 1;

fn init_heap1() {
    let heap_start = unsafe { symbols::HEAP1_TOP as *mut u8 };
    unsafe { ALLOCATOR.lock().init(heap_start, HEAP1_SIZE) }
}

#[unsafe(no_mangle)]
extern "C" fn start(hartid: usize, fdt_ptr: usize) -> ! {
    println!("\n\n\n^w^ welcome to my operating system");
    writer::init_log();

    log::debug!("KERNEL STARTING ON HART#{hartid}");

    let balloc_addr = unsafe { symbols::HEAP0_TOP };
    let balloc = allocator::BitMapAlloc::init(balloc_addr);
    init_heap1();

    // safety: the fdt_ptr needs to be valid. this is "guaranteed" by OpenSBI
    let fdt = unsafe { fdt::Fdt::from_ptr(fdt_ptr as *const u8) }.expect("could not parse fdt");

    let mut balloc = balloc.lock();
    let mut mapper = vmem::init(&mut balloc);

    // map the kernel, stack and the heap onto the memory
    map_vitals(&mut mapper).expect("could not map vital memory");

    // work in progress driver, redundent unless we wanna add multiple serial outputs
    CharDriver::init(fdt, &mut mapper).expect("could not init uart driver");

    // we setup pcie subsystem along with some basic drivers
    let _pci = PciSubsystem::init(fdt, &mut mapper).expect("could not initialise PCI");
    setup_pcie(fdt, &mut mapper);

    #[cfg(test)]
    test_main();

    kinit::pre_kinit(&mut balloc, fdt);
    kinit::kinit(hartid);
}

fn map_vitals(mapper: &mut Mapper) -> Result<(), vmem::MapError> {
    let kernel_start = unsafe { symbols::MEMTOP };

    // we are rounding up the etext
    let etext = round_up_by(unsafe { symbols::ETEXT }, PAGE_SIZE);
    let kernel_pages = (etext - kernel_start) / 4096;

    let heap1 = unsafe { symbols::HEAP1_TOP };
    let heap1_pages = HEAP1_SIZE / PAGE_SIZE;

    let stack_heap0_size = heap1 - etext;
    let stack_heap0_pages = stack_heap0_size / PAGE_SIZE;

    // MAP THE KERNEL
    mapper.map(kernel_start, kernel_start, Perms::EXEC, kernel_pages)?;

    // TODO: map the heap pages during allocation

    // MAP STACK AND HEAP0
    mapper.map(etext, etext, Perms::READ_WRITE, stack_heap0_pages)?;
    // MAP HEAP1
    mapper.map(heap1, heap1, Perms::READ_WRITE, heap1_pages)?;

    Ok(())
}

fn setup_pcie(fdt: fdt::Fdt, mapper: &mut Mapper) {
    // first we fetch the base address of the pcie configuration interface
    let ecam = pci::init(fdt, mapper).expect("could not initialise pci");

    // then we create a manager to handle initialisation of PCIe drivers
    let mut pcie_manager = PcieManager::new(ecam);

    // the following section needs a callback that ANY driver can call from the kernel
    let mut driver = virtio::BlkDriver::new();
    pcie_manager.register_driver(&mut driver);

    pcie_manager.init_drivers(fdt);
}

#[inline]
pub fn round_up_by(input: usize, alignment: usize) -> usize {
    let boundry = alignment - 1;
    (input + boundry) & !boundry
}

#[inline]
pub fn round_down_by(input: usize, alignment: usize) -> usize {
    let boundry = alignment - 1;
    input & !boundry
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

    // we can grep this in target/serial.log to determine if tests were successful or not
    println!("\n=====| ALL TESTS PASSED |=====");
    system_reset(ResetType::Shutdown, ResetReason::None);
    riscv::pauseloop();
}

// ========= ASSEMBLY IMPORTS =========
include_asm!("kernelvec.s");
include_asm!("entry.s");
// ====================================
