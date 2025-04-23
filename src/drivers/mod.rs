#![allow(unused)]

pub mod uart;

pub trait Driver {
    fn init(fdt: fdt::Fdt);
    fn inithart();
}
