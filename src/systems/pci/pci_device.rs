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

    pub fn read_bar(&self, bar_nr: u8) -> u32 {
        let offset = super::OFFSET_BARS[bar_nr as usize];
        self.ecam.read(offset)
    }

    pub fn write_bar(&self, bar_nr: u8, value: u32) {
        let offset = super::OFFSET_BARS[bar_nr as usize];
        self.ecam.write(offset, value);
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

    pub fn get_bar_size(&self, bar_nr: u8) -> (bool, u32) {
        let original: u32 = self.read_bar(bar_nr);

        self.write_bar(bar_nr, u32::MAX);
        let new_value = self.read_bar(bar_nr);

        self.write_bar(bar_nr, original);

        let is_pio = (new_value & 1) != 0;
        assert!(!is_pio, "RISC-V does not support PIO");

        let type_bits = (new_value >> 1) & 0b11;
        let is_64_bits = match type_bits {
            0x0 => false,
            0x2 => true,
            _ => unreachable!(),
        };

        // the first few bits are for conveying info to us, the os
        (is_64_bits, !(new_value & 0xFFFFFFF0) + 1)
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
