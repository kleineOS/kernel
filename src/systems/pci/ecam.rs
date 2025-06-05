use super::pci_device::{Device, DeviceHeader};

#[derive(Debug)]
pub struct EcamLocked {
    ecam: Ecam,
    bus: u8,
    device: u8,
    func: u8,
}

impl EcamLocked {
    // pub fn base_address(&self) -> usize {
    //     self.ecam.address(self.bus, self.device, self.func, 0)
    // }

    pub fn read<T>(&self, offset: u8) -> T {
        self.ecam.read(self.bus, self.device, self.func, offset)
    }

    pub fn write<T>(&self, offset: u8, value: T) {
        self.ecam
            .write(self.bus, self.device, self.func, offset, value);
    }

    // internal function, will not be accessible outside this module
    // this essentially gurantees that this struct was produced from a valid source (device's ecam)
    fn init(ecam: Ecam, bus: u8, device: u8, func: u8) -> Self {
        Self {
            ecam,
            bus,
            device,
            func,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Ecam {
    base_addr: usize,
}

impl Ecam {
    pub(super) fn init(base_addr: usize) -> Self {
        Self { base_addr }
    }

    const fn address(&self, bus: u8, device: u8, func: u8, offset: u8) -> usize {
        self.base_addr
            + ((bus as usize) << 20)
            + ((device as usize) << 15)
            + ((func as usize) << 12)
            + offset as usize
    }

    fn read<T>(&self, bus: u8, device: u8, func: u8, offset: u8) -> T {
        let address = self.address(bus, device, func, offset);
        unsafe { core::ptr::read_volatile(address as *const T) }
    }

    fn write<T>(&self, bus: u8, device: u8, func: u8, offset: u8, value: T) {
        let address = self.address(bus, device, func, offset);
        unsafe { core::ptr::write_volatile(address as *mut T, value) };
    }

    pub(super) fn get_device(&self, bus: u8, device: u8, func: u8) -> Option<Device> {
        let ecam = EcamLocked::init(*self, bus, device, func);
        let header = self.read::<DeviceHeader>(bus, device, func, super::OFFSET_VENDOR_ID);

        match header.vendor_id {
            0xFFFF => None,
            _ => Some(Device { ecam, header }),
        }
    }
}
