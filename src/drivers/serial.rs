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
    fn uart_hex(x: u32);
}

pub fn init() {
    unsafe { uart_init(); }

}

pub fn write(str: &str) {
    for &c in str.as_bytes() {
        unsafe {
            if c == b'\n' {
                uart_send(b'\r');
            }
            uart_send(c);
        }
    }
}

pub struct Console;

impl fmt::Write for Console {
    fn write_str(&mut self, str: &str) -> fmt::Result {
        write(str);
        Ok(())
    }
}

macro_rules! print {
    ($($arg:tt)*) => {{
        core::fmt::write(
            &mut $crate::drivers::serial::Console,
            format_args!($($arg)*),
        ).unwrap();
    }};
}
pub(crate) use print;

macro_rules! println {
    ($($arg:tt)*) => {{
        $crate::drivers::serial::print!($($arg)*);
        $crate::drivers::serial::write("\n");
    }};
}
pub(crate) use println;

pub fn write_hex(x: u32) {
    unsafe { uart_hex(x); }
}

pub fn read_char() -> u8 {
    unsafe { uart_getc() }
}

pub fn write_char(c: u8) {
    unsafe { uart_send(c); }
}

const DEL: u8 = b'\x7F';
const BACKSPACE: u8 = b'\x08';

pub fn backspace() {
    write_char(BACKSPACE);
    write_char(b' ');
    write_char(BACKSPACE);
}

pub fn read_line_utf8() -> Vec<u8> {
    let mut chars = Vec::new();
    loop {
        let c = read_char();
        write_char(c);

        match c {
            b'\n' => break,
            DEL | BACKSPACE => {
                backspace();
                _ = chars.pop();
            },
            _ => chars.push(c),
        }
    }
    chars
}

pub fn read_line() -> String {
    unsafe { String::from_utf8_unchecked(read_line_utf8()) }
}
