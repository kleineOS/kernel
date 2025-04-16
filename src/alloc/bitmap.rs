// use core::sync::atomic::{AtomicBool, Ordering};

use crate::PAGE_SIZE;

/// A simple bitmal data structure that can store up to N * 8 bits of data. It uses smart bitwise
/// operations to store more data than a simple array of bools. It cannot represent any more data,
/// and it cannot grow. It also cannot be created twice
#[derive(Debug)]
pub struct BitMap<const N: usize = PAGE_SIZE> {
    inner: *mut [u8; N],
}

impl BitMap {
    pub fn zeroed(addr: usize) -> Self {
        // commented out to make this data structure more generic
        // static TOGGLE: AtomicBool = AtomicBool::new(false);
        // TOGGLE
        //     .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        //     .expect("multiple invocations of BitMap::zero is not supported");

        let inner = addr as *mut [u8; PAGE_SIZE];

        unsafe { core::ptr::write_bytes(inner, 0, PAGE_SIZE) };

        Self { inner }
    }

    pub fn get(&self, pos: usize) -> bool {
        let index = pos / 8;
        let offset = pos % 8;

        let value = unsafe { (*self.inner)[index] >> offset & 1 };
        assert!(value == 1 || value == 0);

        value == 1
    }

    // we dont "need" mut here, but we *are* mutating so I am going to be explicit
    pub fn put(&mut self, pos: usize, value: bool) {
        assert!(pos < PAGE_SIZE * size_of::<u8>());

        let index = pos / 8;
        let offset = pos % 8;

        unsafe {
            if value {
                (*self.inner)[index] |= 1 << offset;
            } else {
                (*self.inner)[index] &= !(1 << offset);
            }
        }
    }

    /// displays the "index" value of the bitmap (pos / 8). this results in 8 total bits being
    /// displayed in logical order (the physical order is reversed). mostly just for debugging
    pub fn display_chunk(&self, pos: usize) {
        let index = pos / 8;

        let value = unsafe { (*self.inner)[index] };
        let value = value.reverse_bits();

        crate::println!("{value:#08b}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr() -> usize {
        unsafe { crate::HEAP_TOP }
    }

    #[test_case]
    fn test_zero() {
        let bm = BitMap::zeroed(addr());

        for i in 0..PAGE_SIZE {
            unsafe { assert_eq!((*bm.inner)[i], 0, "index#{i}") };
        }
    }

    #[test_case]
    fn test_get_put() {
        let mut bm = BitMap::zeroed(addr());

        // initial state should be all 0s (aka false)
        assert!(!bm.get(0));

        bm.put(13, true);
        assert!(bm.get(13));

        bm.put(13, false);
        assert!(!bm.get(13));
    }
}
