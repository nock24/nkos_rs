use core::fmt;
use alloc::{
    string::String,
    vec::Vec,
};

#[allow(unused_imports)]
unsafe extern "C" {
    fn uart_init();
    fn uart_send(c: u8);
    fn uart_getc() -> u8;
    fn uart_puts(s: *const u8);
}

pub fn init() {
    unsafe { uart_init(); }

}

pub fn write(s: &str) {
    for &c in s.as_bytes() {
        unsafe {
            if c == b'\n' {
                uart_send(b'\r');
            }
            uart_send(c);
        }
    }
}

macro_rules! print {
    ($($arg:tt)*) => {{
        let formatted = format!($($arg)*);
        $crate::drivers::serial::write(formatted.as_str());
    }};
}
pub(crate) use print;

macro_rules! println {
    () => {{
        $crate::drivers::serial::write("\n");
    }};
    ($fmt:expr) => {{
        let formatted = format!(concat!($fmt, "\n"));
        $crate::drivers::serial::write(formatted.as_str());
    }};
    ($fmt:expr, $($arg:tt)*) => {{
        let formatted = format!(concat!($fmt, "\n"), $($arg)*);
        $crate::drivers::serial::write(formatted.as_str());
    }};
}
pub(crate) use println;

pub fn read_char() -> u8 {
    unsafe { uart_getc() }
}

const DEL: u8 = b'\x7F';
const BACKSPACE: u8 = b'\x08';

pub fn read_line_raw() -> Vec<u8> {
    let mut chars = Vec::new();
    loop {
        let c = read_char();

        match c {
            b'\n' => break,
            DEL | BACKSPACE => {
                unsafe {
                    uart_send(BACKSPACE);
                    uart_send(BACKSPACE);
                }
                _ = chars.pop();
            },
            _ => chars.push(c),
        }
    }
    chars
}

pub fn read_line() -> String {
    unsafe { String::from_utf8_unchecked(read_line_raw()) }
}
