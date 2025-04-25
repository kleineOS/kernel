#![allow(unused)]

pub mod uart;

use crate::vmem::Mapper;

pub trait Driver: Sized {
    fn init(fdt: fdt::Fdt, mapper: &mut Mapper) -> Result<Self, DriverError>;
    fn inithart();
}

pub trait CharDriver: Driver {
    fn put_char(&self, c: char);
}

#[derive(Debug)]
pub enum DriverError {
    DeviceNotFound,
}

impl core::fmt::Display for DriverError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}

impl core::error::Error for DriverError {}
