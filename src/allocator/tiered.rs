//! A tiered bitmap allocator
//!
//! or maybe a buddy
//!
//! This is used for more granular allocations.
//!
//! TODO: this is still a work in progress

use crate::PAGE_SIZE;

use super::AllocatorError;

pub struct BuddyTree {
    left: *mut BuddyTree,
    right: *mut BuddyTree,
    alloc_addr: usize,
}

// # Buddy system
// Allocating 32M and 64M for example
//
// |              256M             |
// |      128      |      128      |
// |   64  |   64  |   64  |#######|
// |###|32 |32 |32 |32 |32 |#######|

pub struct BuddySystem {
    base: usize,
}

impl BuddySystem {
    /// # Safety
    /// This is safe as long as the `addr` is valid
    pub unsafe fn init(base: usize) -> Result<Self, AllocatorError> {
        if base % PAGE_SIZE != 0 {
            return Err(AllocatorError::AddrNotAligned {
                addr: base,
                align: PAGE_SIZE,
            });
        }

        Ok(BuddySystem { base })
    }

    pub fn alloc(&mut self, bytes: usize) -> Result<usize, AllocatorError> {
        if !bytes.is_power_of_two() {
            return Err(AllocatorError::InvalidSize { size: bytes });
        }

        Ok(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::symbols;

    #[test_case]
    fn basic_asserts() {
        let buddy = unsafe { BuddySystem::init(symbols::HEAP1_TOP) };
        let mut buddy = buddy.unwrap();
        assert!(buddy.alloc(3).is_err());
        assert!(buddy.alloc(2).is_ok());
    }
}
