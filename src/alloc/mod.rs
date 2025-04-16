#![allow(unused)]

//! A simple bitmap allocator

use spin::Mutex;

const PAGE_SIZE: usize = 4096;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemStatus {
    Free,
    Taken,
}

#[derive(Debug, Clone, Copy)]
pub struct BitMap {
    pub(crate) raw_bitmap: *mut [u8; PAGE_SIZE],
    cursor: usize,
}

impl Iterator for BitMap {
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
    pub(crate) bitmap: BitMap,
}

impl BitMapAlloc {
    pub fn init() -> Mutex<Self> {
        let cursor = 0;
        let raw_bitmap = unsafe {
            let raw = crate::HEAP_TOP as *mut [u8; PAGE_SIZE];
            *raw = core::mem::zeroed();
            raw
        };

        let bitmap = BitMap { raw_bitmap, cursor };
        Mutex::new(Self { bitmap })
    }

    /// allocate the given number of contigous pages
    pub fn alloc(&mut self, num_pages: usize) -> usize {
        assert!(num_pages > 0, "Cannot allocate zero pages");

        let mut contigous_count = 0;
        let mut start_index = 0;
        let mut loc = None;

        // Reset bitmap cursor for fresh search
        self.bitmap.cursor = 0;

        for (index, state) in self.bitmap.enumerate() {
            match state {
                MemStatus::Free => {
                    if contigous_count == 0 {
                        start_index = index;
                    }
                    contigous_count += 1;

                    if contigous_count >= num_pages {
                        loc = Some(start_index);
                        break;
                    }
                }
                MemStatus::Taken => {
                    contigous_count = 0;
                }
            }
        }

        let offset_multiplier = loc.expect("out of space on the bitmap") + 1;

        // Mark all allocated pages as taken
        for i in 0..num_pages {
            let page_index = offset_multiplier + i;
            let index = page_index / 8;
            let offset = page_index % 8;

            unsafe {
                let curr = (*self.bitmap.raw_bitmap)[index];
                (*self.bitmap.raw_bitmap)[index] = curr | (1 << offset);
            }
        }

        let heap_top = unsafe { crate::HEAP_TOP };
        heap_top + (offset_multiplier * PAGE_SIZE)
    }

    pub fn free(&mut self, addr: usize, num_pages: usize) {
        assert!(num_pages > 0, "Cannot free zero pages");

        let top = unsafe { crate::HEAP_TOP };
        let offset_multiplier = (addr - top) / PAGE_SIZE;

        for i in 0..num_pages {
            let page_index = offset_multiplier + i;
            let index = page_index / 8;
            let offset = page_index % 8;

            unsafe {
                let curr = (*self.bitmap.raw_bitmap)[index];
                (*self.bitmap.raw_bitmap)[index] = curr & !(1 << offset);
            }
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
