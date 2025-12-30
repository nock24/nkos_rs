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
    ($str:expr) => {{
        let formatted = format!(concat!($str, "\n"));
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

pub fn write_char(c: u8) {
    unsafe { uart_send(c) }
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
