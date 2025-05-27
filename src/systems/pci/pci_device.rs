#[derive(Debug)]
pub struct Device {
    pub ecam: super::ecam::EcamLocked,
    pub header: DeviceHeader,
}

impl Device {
    pub fn vendor_id(&self) -> u16 {
        self.header.vendor_id
    }

    pub fn device_id(&self) -> u16 {
        self.header.device_id
    }

    pub fn header_type(&self) -> HeaderType {
        self.header.header_type
    }

    pub fn disable_io_space(&self) {
        let mut cmd = self.header.command;
        cmd &= 0b0; // we turn OFF bit 0
        self.ecam.write(super::OFFSET_COMMAND, cmd);
    }

    // RISC-V does not support PIO, enable_io_space will not be available

    pub fn disable_mem_space(&self) {
        let mut cmd = self.header.command;
        cmd &= 0b01; // we turn OFF bit 1
        self.ecam.write(super::OFFSET_COMMAND, cmd);
    }

    pub fn enable_mem_space(&self) {
        let mut cmd = self.header.command;
        cmd |= 0b10; // we turn ON bit 1
        self.ecam.write(super::OFFSET_COMMAND, cmd);
    }

    pub fn get_capabilities<T, V: Extend<T>>(&self, list: &mut V) {
        let offset = match self.header.header_type {
            HeaderType::Pci2Cardbus => 0x14,
            _ => 0x34,
        };

        let pointer = self.ecam.read::<u8>(offset);
        let cap_ptr = if pointer != 0 { Some(pointer) } else { None };

        self.enum_capabilities(cap_ptr, list);
    }

    /// # Returns
    /// -> (address, prefetchable, is_64_bit, is_pio)
    /// HACK: ideally I return an Enum, even if PIO is not supported
    pub fn get_bar_size(&self, bar_nr: u8) -> (u32, bool, bool, bool) {
        assert!(bar_nr <= 5, "the bar_nr provided is too big (max 5)");
        let bar_offset = super::OFFSET_BARS[bar_nr as usize];

        let value = {
            let original: u32 = self.ecam.read(bar_offset);
            self.ecam.write(bar_offset, u32::MAX);
            let value: u32 = self.ecam.read(bar_offset);
            self.ecam.write(bar_offset, original);
            value
        };

        log::info!("{value:#x}");

        let is_pio = (value & 0x1) > 0;
        if !is_pio {
            // Memory space BAR
            let type_bits = (value & 0x6) >> 1; // bits 2:1
            let is_64_bit = type_bits == 0x2;
            assert!(is_64_bit || (type_bits == 0x0), "must be 0x2 or 0x0");
            let prefetchable = ((value & 0x8) >> 3) > 0; // bit 3
            let address = value & 0xFFFFFFF0; // clear lower 4 bits
            (address, prefetchable, is_64_bit, is_pio)
        } else {
            panic!("PIO is not supported on RISC-V");
        }
    }

    fn enum_capabilities<T, V: Extend<T>>(&self, ptr: Option<u8>, list: &mut V) {
        if let Some(ptr) = ptr {
            let cap = self.ecam.read::<super::Capabilities<T>>(ptr);

            list.extend(Some(cap.data));

            if cap.next_cap != 0 {
                self.enum_capabilities(Some(cap.next_cap), list);
            }
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct DeviceHeader {
    pub vendor_id: u16,
    pub device_id: u16,
    pub command: u16,
    pub status: u16,
    pub revision_id: u8,
    pub prog_if: u8,
    pub subclass: u8,
    pub class_code: u8,
    pub cache_line_size: u8,
    pub latency_timer: u8,
    pub header_type: HeaderType,
    pub bist: u8,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum HeaderType {
    GeneralDevice = 0,
    Pci2Pci = 1,
    Pci2Cardbus = 2,
}
