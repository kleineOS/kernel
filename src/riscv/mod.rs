#![allow(unused)]

//! # Wrappers for common RISC-V instructions
//! These mostly wrap around raw assembly

mod frame;
pub mod sbi;

use core::arch::asm;

pub use frame::*;

/// `PAUSE` instruction wrapper
///
/// NOTE: requires Zihintpause extension to properly function, and does nothing if not present
///
/// Provides a hint to the implimentation that the current hart's rate of instruction returement
/// should be temoorarily reduced or paused. The duration of its effect must be bounded and may be
/// zero
#[inline]
pub fn pause() {
    if cfg!(target_feature = "zihintpause") {
        unsafe { asm!("pause", options(nomem, nostack)) };
    }
}

#[inline]
pub fn pauseloop() -> ! {
    loop {
        pause();
    }
}

/// `WFI` instruction wrapper
///
/// Provides a hint to the implementation that the current hart can be stalled until an interrupt might need servicing.
/// The WFI instruction is just a hint, and a legal implementation is to implement WFI as a NOP.
pub fn wfi() {
    unsafe { asm!("wfi", options(nomem, nostack)) };
}

/// `TIME` instruction wrapper
pub fn time() -> usize {
    unsafe {
        let time: usize;
        asm!("csrr {}, time", out(reg) time, options(nomem, nostack));
        time
    }
}

// flush the TLB
pub fn sfence_vma() {
    // zero zero means all tlb entries
    unsafe { asm!("sfence.vma zero, zero", options(nomem, nostack)) };
}

pub mod satp {
    use super::*;

    pub fn write(value: usize) {
        unsafe { asm!("csrw satp, {}", in(reg) value, options(nomem, nostack)) };
    }

    pub fn read() -> usize {
        unsafe {
            let satp: usize;
            asm!("csrr {}, satp", out(reg) satp, options(nomem, nostack));
            satp
        }
    }
}

pub mod interrupt {
    use super::*;

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(usize)]
    pub enum Trap {
        Interrupt(Interrupt),
        Exception(Exception),
    }

    #[allow(clippy::enum_variant_names)]
    #[repr(usize)]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum Interrupt {
        SupervisorSoft = 1,
        SupervisorTimer = 5,
        SupervisorExternal = 9,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(usize)]
    pub enum Exception {
        InstructionMisaligned = 0,
        InstructionFault = 1,
        IllegalInstruction = 2,
        Breakpoint = 3,
        LoadMisaligned = 4,
        LoadFault = 5,
        StoreMisaligned = 6,
        StoreFault = 7,
        UserEnvCall = 8,
        SupervisorEnvCall = 9,
        InstructionPageFault = 12,
        LoadPageFault = 13,
        StorePageFault = 15,
    }

    /// Retrieves the cause of a trap in the current hart (supervisor mode).
    #[inline]
    pub fn cause() -> Trap {
        let scause = unsafe {
            let scause: usize;
            asm!("csrr {}, scause", out(reg) scause, options(nomem, nostack));
            scause
        };

        let int = ((scause >> 63) & 1) == 1;
        let cause = scause & ((1 << 63) - 1);

        if int {
            let interrupt = unsafe { core::mem::transmute::<usize, Interrupt>(cause) };
            Trap::Interrupt(interrupt)
        } else {
            let exception = unsafe { core::mem::transmute::<usize, Exception>(cause) };
            Trap::Exception(exception)
        }
    }

    /// Enables all the interrupts in the current hart (supervisor mode).
    /// # Safety
    /// Do not call this function inside a critical section.
    #[inline]
    pub unsafe fn enable_all() {
        unsafe { asm!("csrw sie, {}", in(reg) 1 << 5 | 1 << 9, options(nomem, nostack)) };
    }

    /// Disables all interrupts in the current hart (supervisor mode).
    #[inline]
    pub fn disable() {
        // TODO: I want to instead have some closure like syntax to disable interrupts for only a
        // given function
        unsafe { asm!("csrw sie, {}", in(reg) 0, options(nomem, nostack)) };
    }
}

#[macro_export]
macro_rules! include_asm {
    ($file:expr $(,)?) => {
        core::arch::global_asm!(include_str!($file));
    };
}
