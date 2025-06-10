#![allow(unused)]

use crate::allocator::BitMapAlloc;
use crate::riscv::Frame;

pub struct Process {
    trap_frame: Frame,
}

pub fn init(balloc: &mut BitMapAlloc) {}

pub fn spawn() {}
