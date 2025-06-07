use crate::drivers::regcell::*;

#[derive(Debug)]
pub(super) struct BlkConfig {
    #[allow(unused)]
    pub(super) inner: *mut BlkConfigRaw,
}

impl BlkConfig {
    pub unsafe fn from_raw(addr: usize) -> Self {
        let inner = addr as *mut BlkConfigRaw;
        Self { inner }
    }
}

#[repr(C)]
#[derive(Debug)]
pub(super) struct BlkConfigRaw {
    capacity: RegCell<u64, RW>,
    size_max: RegCell<u32, RW>,
    seg_max: RegCell<u32, RW>,

    geometry: BlkGeometry,

    blk_size: RegCell<u32, RW>,

    topology: BlkTopology,

    writeback: RegCell<u8, RW>,
    _unused0: RegCell<u8, RW>,
    num_queues: RegCell<u16, RW>,
    max_discard_sectors: RegCell<u32, RW>,
    max_discard_seg: RegCell<u32, RW>,
    discard_sector_alignment: RegCell<u32, RW>,
    max_write_zeroes_sectors: RegCell<u32, RW>,
    max_write_zeroes_seg: RegCell<u32, RW>,
    write_zeroes_may_unmap: RegCell<u8, RW>,
    _unused1: [u8; 3],
    max_secure_erase_sectors: RegCell<u32, RW>,
    max_secure_erase_seg: RegCell<u32, RW>,
    secure_erase_sector_alignment: RegCell<u32, RW>,

    // === virtio_blk_zoned_characteristics
    zoned: BlkZonedCharacteristics,
}

#[repr(C)]
#[derive(Debug)]
struct BlkGeometry {
    geom_cylinders: RegCell<u16, RW>,
    geom_heads: RegCell<u8, RW>,
    geom_sectors: RegCell<u8, RW>,
}

#[repr(C)]
#[derive(Debug)]
struct BlkTopology {
    // more fields need to go here (might split it in a new file)
    physical_block_exp: RegCell<u8, RW>,
    // offset of first aligned logical block
    alignment_offset: RegCell<u8, RW>,
    // suggested minimum I/O size in blocks
    min_io_size: RegCell<u16, RW>,
    // optimal (suggested maximum) I/O size in blocks
    opt_io_size: RegCell<u32, RW>,
}

#[repr(C)]
#[derive(Debug)]
struct BlkZonedCharacteristics {
    zone_sectors: RegCell<u32, RW>,
    max_open_zones: RegCell<u32, RW>,
    max_active_zones: RegCell<u32, RW>,
    max_append_sectors: RegCell<u32, RW>,
    write_granularity: RegCell<u32, RW>,
    model: RegCell<u8, RW>,
    unused2: [u8; 3],
}
