#![allow(unused)]

//! (to be moved)
//! A simple bitmap allocator

mod bitmap;

use spin::Mutex;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemStatus {
    Free,
    Taken,
}

#[derive(Debug, Clone, Copy)]
pub struct BitMapO {
    pub(crate) raw_bitmap: *mut [u8; crate::PAGE_SIZE],
    cursor: usize,
}

impl Iterator for BitMapO {
    type Item = MemStatus;

    fn next(&mut self) -> Option<Self::Item> {
        let bmlen = (unsafe { *self.raw_bitmap }).len();
        // the bitmap being full results in a issues anyways, no point returning None
        assert!(self.cursor < bmlen * 8, "the bitmap is full");

        let index = self.cursor / 8;
        let offset = self.cursor % 8;

        let offset_val = unsafe { (*self.raw_bitmap)[index] >> offset & 1 };
        assert!(
            offset_val == 0 || offset_val == 1,
            "heap is corrupted. found value {offset_val} when expected 0 or 1",
        );

        self.cursor += 1;

        if offset_val == 1 {
            Some(MemStatus::Taken)
        } else {
            Some(MemStatus::Free)
        }
    }
}

#[derive(Debug)]
pub struct BitMapAlloc {
    // this can store info on 4096*8 32768 pages
    // in total, this represents ~130M of memory
    pub(crate) bitmap: bitmap::BitMap,
}

impl BitMapAlloc {
    pub fn init() -> Mutex<Self> {
        let addr = unsafe { crate::HEAP_TOP };
        let bitmap = bitmap::BitMap::zeroed(addr);

        Mutex::new(Self { bitmap })
    }

    /// allocate the given number of contigous pages
    pub fn alloc(&mut self, num_pages: usize) -> usize {
        assert!(num_pages > 0, "Cannot allocate zero pages");

        todo!()
    }

    pub fn free(&mut self, addr: usize, num_pages: usize) {
        assert!(num_pages > 0, "Cannot free zero pages");

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_balloc() {
        let top = unsafe { crate::HEAP_TOP };
        let balloc = BitMapAlloc::init();

        let location = balloc.lock().alloc(4);
        assert_eq!(location, top + 0x1000);

        let location = balloc.lock().alloc(6);
        assert_eq!(location, top + 0x6000);

        // we try freeing, so we can allocate again on the same spot
        balloc.lock().free(location, 6);

        let location = balloc.lock().alloc(4);
        assert_eq!(location, top + 0x6000);

        let location = balloc.lock().alloc(6);
        assert_eq!(location, top + 0xb000);
    }
}
