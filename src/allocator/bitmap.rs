/// A simple data structure that holds SIZE * 8 bits of data. Stores more data than a simple array
/// of bools by using bitwise operations to fully utilise an 8 bit wide memory address.
///
/// **IMPORTANT**: Can store up to SIZE * 8 "bool" values
///
/// # Usage
/// ```rust
/// let size = 4096;
/// let addr = 0x8000_0000; // must be a valid address
/// let mut bm = unsafe { BitMap::zeroed(addr) };
/// assert_eq!(bm.len(), size * 8);
/// bm.put(123, true);
/// assert_eq!(bm.get(123), true);
/// ```
#[derive(Debug)]
pub struct BitMap<const SIZE: usize> {
    inner: *mut [u8; SIZE],
}

impl<const SIZE: usize> BitMap<SIZE> {
    pub const fn len(&self) -> usize {
        SIZE * u8::BITS as usize
    }

    /// Create a new [BitMap] at the given address, with all bits flipped to 0
    /// # Safety
    /// safe, as long as the `addr` is valid
    pub unsafe fn zeroed(addr: usize) -> Self {
        let inner = addr as *mut [u8; SIZE];

        // safety: this is safe as long as the address of inner is valid
        unsafe { core::ptr::write_bytes(inner, 0, SIZE) };

        Self { inner }
    }

    /// Get a bit from the given position
    pub fn get(&self, pos: usize) -> bool {
        let index = pos / 8;
        let offset = pos % 8;

        // safety: this is safe as long as self.inner is valid
        let value = unsafe { (*self.inner)[index] >> offset & 1 };
        assert!(value == 1 || value == 0);

        value == 1
    }

    /// Flip a bit in the given position to your desired value
    pub fn put(&mut self, pos: usize, value: bool) {
        assert!(pos < SIZE * size_of::<u8>(), "bitmap is full");

        let index = pos / 8;
        let offset = pos % 8;

        // safety: this is safe as long as self.inner is valid
        unsafe {
            if value {
                (*self.inner)[index] |= 1 << offset;
            } else {
                (*self.inner)[index] &= !(1 << offset);
            }
        }
    }

    // displays the "index" value of the bitmap (pos / 8). this results in 8 total bits being
    // displayed in logical order (the physical order is reversed). mostly just for debugging
    // pub fn display_chunk(&self, pos: usize) {
    //     let index = pos / 8;
    //
    //     // safety: this is safe as long as self.inner is valid
    //     let value = unsafe { (*self.inner)[index] };
    //     let value = value.reverse_bits();
    //
    //     crate::println!("{value:#08b}");
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PAGE_SIZE, symbols};

    #[test_case]
    fn test_zero() {
        const N: usize = 1234;
        let bm = unsafe { BitMap::<N>::zeroed(symbols::HEAP0_TOP) };

        for i in 0..N {
            unsafe { assert_eq!((*bm.inner)[i], 0, "index#{i}") };
        }

        // and a larger, the "default" PAGE_SIZE amount
        let bm = unsafe { BitMap::<PAGE_SIZE>::zeroed(symbols::HEAP0_TOP) };

        for i in 0..PAGE_SIZE {
            unsafe { assert_eq!((*bm.inner)[i], 0, "index#{i}") };
        }
    }

    #[test_case]
    fn test_get_put() {
        let mut bm = unsafe { BitMap::<PAGE_SIZE>::zeroed(symbols::HEAP0_TOP) };

        // initial state should be all 0s (aka false)
        assert!(!bm.get(0));

        bm.put(13, true);
        assert!(bm.get(13));

        bm.put(13, false);
        assert!(!bm.get(13));
    }

    #[test_case]
    fn test_len() {
        let bm = unsafe { BitMap::<PAGE_SIZE>::zeroed(symbols::HEAP0_TOP) };
        assert_eq!(bm.len(), PAGE_SIZE * 8);
    }
}
