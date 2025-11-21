#![allow(unused)]

use core::alloc::Layout;
use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::vec::Vec;
use spin::{Lazy, Mutex};

use crate::allocator::BitMapAlloc;
use crate::riscv::Frame;

const STACK_LAYOUT: Layout = unsafe { Layout::from_size_align_unchecked(4096, 4096) };
static NEXT_PID: AtomicUsize = AtomicUsize::new(0);

type LMVec<T> = Lazy<Mutex<Vec<T>>>;

/// A list of all the processes
static PROCLIST: LMVec<Process> = Lazy::new(|| Mutex::new(Vec::with_capacity(8)));
static READY_LIST: LMVec<Process> = Lazy::new(|| Mutex::new(Vec::with_capacity(8)));

struct Process {
    pid: usize,
    trap_frame: Frame,
}

// spawn a process in s-mode
pub fn k_spawn(f: fn()) {
    let pid = NEXT_PID.fetch_add(1, Ordering::Relaxed);

    const STACK_LAYOUT: Layout = unsafe { Layout::from_size_align_unchecked(4096, 4096) };

    let stack_top = {
        let stack_pages = crate::STACK_PAGES;
        let stack_addr = unsafe { alloc::alloc::alloc(STACK_LAYOUT) } as usize;
        let stack_size = stack_pages * crate::PAGE_SIZE;
        stack_addr + stack_size
    };
}

#[unsafe(no_mangle)]
extern "C" fn task_end() {
    panic!("task ended when it shouldnt have");
}
