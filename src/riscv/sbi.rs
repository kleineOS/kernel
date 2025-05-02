#[derive(Debug, Default)]
struct Args {
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
}

#[derive(Debug)]
struct EcallRet {
    a: usize,
    b: usize,
}

fn ecall(mut args: Args, fid: usize, eid: usize) -> EcallRet {
    let mut a = args.a0;
    let mut b = args.a1;

    unsafe {
        core::arch::asm!("ecall",
            inout("a0") a,
            inout("a1") b,
            in("a2") args.a2,
            in("a3") args.a3,
            in("a4") args.a4,
            in("a5") args.a5,
            in("a6") fid,
            in("a7") eid,
        );
    };

    EcallRet { a, b }
}

pub mod hsm {
    //! # Hart State Management Extension (EID: 0x48534D "HSM")
    //!
    //! The Hart State Management (HSM) Extension introduces a set of hart states and a set of
    //! functions which allow the supervisor-mode software to request a hart state change.
    //!
    //! | State ID | State Name     | Description |
    //! |----------|----------------|-------------|
    //! | 0        | STARTED        | The hart is physically powered up and executing normally |
    //! | 1        | STOPPED        | The hart is not executing in supervisor-mode or any lower privilege mode. It is probably powered-down by the SBI implementation if the underlying platform has a mechanism to physically power-down harts. |
    //! | 3        | START_PENDING  | Some other hart has requested to start (or power-up) the hart from the STOPPED state and the SBI implementation is still working to get the hart in the STARTED state. |
    //! | 4        | STOP_PENDING   | The hart has requested to stop (or power-down) itself from the STARTED state and the SBI implementation is still working to get the hart in the STOPPED state. |
    //! | 5        | SUSPENDED      | This hart is in a platform specific suspend (or low power) state. |
    //! | 6        | RESUME_PENDING | An interrupt or platform specific hardware event has caused the hart to resume normal execution from the SUSPENDED state and the SBI implementation is still working to get the hart in the STARTED state. |

    use super::*;

    const EID: usize = 0x48534D;
    const FID_HART_START: usize = 0;

    pub fn start(hartid: usize, ra: usize, a2: usize) {
        let args = Args {
            a0: hartid,
            a1: ra,
            a2,
            ..Default::default()
        };

        let ret = ecall(args, FID_HART_START, EID);
        //crate::println!("{ret:?}");
    }
}

pub mod time {
    use super::*;
    const EID: usize = 0x54494D45;
    const FID_SET_TIMER: usize = 0;

    pub fn set_timer(cycles: usize) {
        let args = Args {
            a0: cycles,
            ..Default::default()
        };

        ecall(args, FID_SET_TIMER, EID);
    }
}

pub mod dbcn {
    use super::*;
    const EID: usize = 0x4442434E;
    const FID_WRITE: usize = 0;

    pub fn write(string: &str) {
        let args = Args {
            a0: string.len(),
            a1: string.as_ptr() as usize,
            ..Default::default()
        };

        ecall(args, FID_WRITE, EID);
    }
}

pub mod srst {
    //! System Reset Extension (EID #0x53525354 "SRST")

    use super::*;
    const EID: usize = 0x53525354;
    const FID_SYSTEM_RESET: usize = 0;

    #[repr(u32)]
    pub enum ResetType {
        Shutdown = 0,
        ColdReboot = 1,
        WarnReboot = 2,
    }

    #[repr(u32)]
    pub enum ResetReason {
        None = 0,
        Failure = 1,
    }

    pub fn system_reset(reset_type: ResetType, reason: ResetReason) {
        let args = Args {
            a0: reset_type as usize,
            a1: reason as usize,
            ..Default::default()
        };

        ecall(args, FID_SYSTEM_RESET, EID);
    }
}
