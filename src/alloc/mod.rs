#![allow(unused)]

pub fn init() -> &'static mut [u8; 4096] {
    let heap = unsafe { crate::HEAP_TOP } as *mut [u8; 4096];
    let first_page = unsafe { &mut *heap };
    *first_page = [0u8; 4096];
    unsafe { core::ptr::write_bytes(heap, 0, first_page.len()) };

    log::info!("heap initialized at {:#x}", heap as usize);

    first_page
}

pub fn alloc1(table: &mut [u8; 4096]) -> *mut u8 {
    let heap = unsafe { crate::HEAP_TOP };
    let mut next = None;
    for (i, item) in table.iter_mut().skip(1).enumerate() {
        if *item == 0 {
            *item = 1;
            next = Some(i);
            break;
        }
    }

    // TODO: create a new "table" for the next 4096 pages
    let next = next.expect("no free space");
    let offset = 0x1000 * next;

    // this is where we give the user the address
    let next_heap = heap + offset;

    next_heap as *mut u8
}
