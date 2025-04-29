use core::alloc::GlobalAlloc;
use core::ptr::null_mut;
use core::sync::atomic::AtomicPtr;

use crate::PAGE_SIZE;

use super::BitMapAlloc;

static ALLOC_PTR: AtomicPtr<InnerAlloc> = AtomicPtr::new(null_mut());

pub struct InnerAlloc;

/// global bitmap alloc
pub struct GBMAlloc;

// # Buddy system
// |               256M              |
// |                |                |
// |       |        |       |        |
// |   |   |    |   |   |   |    |   |

unsafe impl GlobalAlloc for GBMAlloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        log::debug!("{layout:?}");
        todo!()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        todo!()
    }
}
