//! A tiered bitmap allocator
//!
//! or maybe a buddy
//!
//! This is used for more granular allocations.

pub struct BuddySystem {
    base: usize,
}

impl BuddySystem {
    /// # Safety
    /// safe, as long as the `addr` is valid
    pub unsafe fn init(base: usize) -> Self {
        BuddySystem { base }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test_case]
//     fn hello() {
//         log::info!("[START] allocator::tiered::tests::hello");
//         let buddy = unsafe { BuddySystem::init(crate::HEAP1_TOP) };
//     }
// }
