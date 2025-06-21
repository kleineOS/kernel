#![allow(unused)]

use alloc::collections::BTreeMap;

use types::*;

pub struct VirtQueue {}

impl VirtQueue {
    pub fn init(virtqueues: BTreeMap<u16, u16>) -> Self {
        // TODO: I will get back to this later
        Self {}
    }
}

mod types {}
