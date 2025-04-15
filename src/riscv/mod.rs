#![allow(unused)]

//! # Wrappers for common RISC-V instructions
//! This mostly wraps around the [::riscv] crate. These wrappers are designed to be easier to write
//! and to be overall integrate nicely with my other stuff

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
    ::riscv::asm::wfi();
}

/// `TIME` instruction wrapper
pub fn time() -> usize {
    ::riscv::register::time::read()
}

pub mod interrupt {
    use ::riscv::interrupt::cause as rv_cause;
    pub use ::riscv::interrupt::{Exception, Interrupt, Trap};

    use super::*;

    /// Retrieves the cause of a trap in the current hart (supervisor mode).
    #[inline]
    pub fn cause() -> Trap<Interrupt, Exception> {
        rv_cause::<Interrupt, Exception>()
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
        ::riscv::interrupt::disable();
    }
}

#[macro_export]
macro_rules! include_asm {
    ($file:expr $(,)?) => {
        core::arch::global_asm!(include_str!($file));
    };
}
