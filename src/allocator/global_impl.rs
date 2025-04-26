use core::alloc::GlobalAlloc;

use crate::PAGE_SIZE;

use super::BitMapAlloc;

/// global bitmap alloc
pub struct GBMAlloc;

unsafe impl GlobalAlloc for GBMAlloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        log::debug!("{layout:?}");
        todo!()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        todo!()
    }
}
