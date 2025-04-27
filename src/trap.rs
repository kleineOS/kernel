use crate::riscv;
use crate::riscv::interrupt::{Exception, Interrupt, Trap};
use crate::riscv::sbi;

/// This is the value that is set in stvec. Loading and saving of registers is handled by the
/// compiler, we dont have to do it manually. Allocates 480 bytes on the stack, and saves in the
/// order that RISC-V spec defines it's registers. (18.2 RVG Calling Convention)
#[unsafe(no_mangle)]
extern "C" fn kerneltrap(frame: *const riscv::Frame) {
    let cause = riscv::interrupt::cause();
    let frame = unsafe { &*frame };

    match cause {
        Trap::Interrupt(interrupt) => handle_interrupt(interrupt),
        Trap::Exception(exception) => handle_exception(exception, frame),
    };
}

fn handle_interrupt(interrupt: Interrupt) {
    match interrupt {
        Interrupt::SupervisorSoft => todo!("software interrupt"),
        Interrupt::SupervisorTimer => reset_timer(),
        Interrupt::SupervisorExternal => todo!("external interrupt"),
    };
}

fn handle_exception(exception: Exception, frame: &riscv::Frame) {
    log::error!("TRAP: SEPC: {:#x}", ::riscv::register::sepc::read());
    log::error!("TRAP: EXCEPTION: {exception:?}");
    frame.pretty_print();

    riscv::pauseloop();
}

fn reset_timer() {
    // log::debug!("timer reset");
    sbi::time::set_timer(riscv::time() + crate::INTERVAL)
}
