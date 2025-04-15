use core::fmt;

use crate::riscv::sbi;

static WRITER: spin::Mutex<Writer> = spin::Mutex::new(Writer);
static LOGGER: WriterLogger = WriterLogger;

// colours for pretty printing
pub const RESET: &str = "\x1b[0m";
pub const LIGHT_CYAN: &str = "\x1b[96m";
pub const GREY: &str = "\x1b[90m";
pub const BRIGHT_MAGENTA: &str = "\x1b[95m";

pub struct Writer;
pub struct WriterLogger;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        sbi::dbcn::write(s);
        Ok(())
    }
}

impl log::Log for WriterLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        crate::println!("{}: {}", record.level(), record.args());
    }

    fn flush(&self) {
        unimplemented!();
    }
}

pub fn init_log() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .expect("could not enable logger");
}

// pub fn clear_screen() {
//     const CLEAR_SCREEN: &str = "\x1b[2J\x1b[1;1H";
//     crate::print!("{CLEAR_SCREEN}");
// }

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write as _;
    ::riscv::interrupt::free(|| WRITER.lock().write_fmt(args).unwrap());
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::writer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
