//! A tiered bitmap allocator
//!
//! or maybe a buddy
//!
//! This is used for more granular allocations.

use crate::PAGE_SIZE;

use super::AllocatorError;

pub struct BuddyTree {
    left: *mut BuddyTree,
    right: *mut BuddyTree,
    alloc_addr: usize,
}

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

    #[test_case]
    fn basic_asserts() {
        let buddy = unsafe { BuddySystem::init(crate::HEAP1_TOP) };
        let mut buddy = buddy.unwrap();
        assert!(buddy.alloc(3).is_err());
        assert!(buddy.alloc(2).is_ok());
    }
}
