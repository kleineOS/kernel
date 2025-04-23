//! (to be moved)
//! A simple bitmap allocator

mod bitmap;
mod global_impl;

use spin::Mutex;

use crate::{HEAP_TOP, PAGE_SIZE};

#[derive(Debug)]
pub struct BitMapAlloc {
    pub(crate) bitmap: bitmap::BitMap<PAGE_SIZE>,
}

impl BitMapAlloc {
    pub fn init() -> Mutex<Self> {
        let addr = unsafe { HEAP_TOP };
        let bitmap = bitmap::BitMap::zeroed(addr);

        Mutex::new(Self { bitmap })
    }

    /// allocate the given number of contigous pages
    pub fn alloc(&mut self, num_pages: usize) -> usize {
        assert!(num_pages > 0, "Cannot allocate zero pages");
        let base_addr = unsafe { HEAP_TOP } + PAGE_SIZE;

        let mut start_idx = None;
        let mut found = 0;

        // it will panic if we go over the limit, and a panic is good for such a scenario
        for i in 0.. {
            let is_free = !self.bitmap.get(i);

            match (is_free, start_idx) {
                // not free, but we had a chain going
                (false, Some(_)) => {
                    start_idx = None;
                    found = 0;
                }
                // free, and we have not found anything yet
                (true, None) => {
                    start_idx = Some(i);
                    found += 1;
                }
                // free, and we are already in a chain
                (true, Some(_)) => found += 1,
                // not free, and we have not found anything yet
                (false, None) => (),
            };

            if found == num_pages {
                break;
            }
        }

        // the expect will probably not trigger, as we panic before that
        let start_idx = start_idx.expect("no free pages found");

        for i in start_idx..(start_idx + num_pages) {
            // we claim the pages over here by setting them to true
            self.bitmap.put(i, true);
        }

        base_addr + (PAGE_SIZE * start_idx)
    }

    pub fn free(&mut self, addr: usize, num_pages: usize) {
        assert!(num_pages > 0, "Cannot free zero pages");
        let base_addr = unsafe { HEAP_TOP } + PAGE_SIZE;

        let idx = (addr - base_addr) / PAGE_SIZE;

        for i in idx..(idx + num_pages) {
            assert!(self.bitmap.get(i), "trying to free an un-allocated page");
            self.bitmap.put(i, false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_balloc() {
        let top = unsafe { crate::HEAP_TOP };
        let balloc = BitMapAlloc::init();

        let alloc0 = balloc.lock().alloc(4);
        assert_eq!(alloc0, top + 0x1000);

        let alloc1 = balloc.lock().alloc(6);
        assert_eq!(alloc1, top + 0x5000);

        let alloc2 = balloc.lock().alloc(1);
        assert_eq!(alloc2, top + 0xb000);

        // we try freeing, so we can allocate again on the same spot
        balloc.lock().free(alloc1, 6);

        let location = balloc.lock().alloc(4);
        assert_eq!(location, top + 0x5000);

        let location = balloc.lock().alloc(6);
        assert_eq!(location, top + 0xc000);
    }
}
