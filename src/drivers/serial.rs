use alloc::{string::String, vec::Vec};
use core::fmt;

#[allow(unused_imports)]
unsafe extern "C" {
    fn uart_init();
    fn uart_send(c: u8);
    fn uart_getc() -> u8;
    fn uart_puts(s: *const u8);
    fn uart_hex(x: u32);
}

pub fn init() {
    unsafe {
        uart_init();
    }
}

pub fn write(str: &[u8]) {
    for &c in str.iter() {
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
        write(str.as_bytes());
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
        $crate::drivers::serial::write(b"\n");
    }};
}
pub(crate) use println;

pub fn write_hex(x: u32) {
    unsafe {
        uart_hex(x);
    }
}

pub fn read_char() -> u8 {
    unsafe { uart_getc() }
}

pub fn write_char(c: u8) {
    unsafe {
        uart_send(c);
    }
}

pub const DEL: u8 = b'\x7F';
pub const BACKSPACE: u8 = b'\x08';

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
            }
            _ => chars.push(c),
        }
    }
    chars
}

pub fn read_line() -> String {
    unsafe { String::from_utf8_unchecked(read_line_utf8()) }
}

pub fn insert_char(ch: u8) {
    // Inserts whitespace.
    write(b"\x1B[@");
    write_char(ch);
}

pub fn delete_char() {
    write(b"\x1B[P");
}

pub fn cursor_left() {
    write(b"\x1B[D");
}

pub fn cursor_right() {
    write(b"\x1B[C");
}

pub fn cursor_up() {
    write(b"\x1B[A");
}

pub fn cursor_down() {
    write(b"\x1B[B");
}

/// pos: (x, y)
pub fn move_cursor(pos: (usize, usize)) {
    print!("\x1B[{};{}H", pos.1 + 1, pos.0 + 1);
}

pub fn cursor_home() {
    move_cursor((0, 0));
}

pub fn block_cursor() {
    write(b"\x1B[2 q");
}

pub fn line_cursor() {
    write(b"\x1B[6 q");
}

pub fn clear() {
    write(b"\x1B[2J");
    cursor_home();
}

pub fn clear_line() {
    write(b"\x1B[2K");
}

/// -> (rows, cols)
pub fn dimensions() -> (usize, usize) {
    write(b"\x1B[18t");
    assert_eq!(read_char(), 0x1B);
    assert_eq!(read_char(), b'[');
    assert_eq!(read_char(), b'8');
    assert_eq!(read_char(), b';');
    let (rows, ch) = parse_u16();
    assert_eq!(ch, b';');
    let (cols, ch) = parse_u16();
    assert_eq!(ch, b't');

    (rows as usize, cols as usize)
}

fn parse_u16() -> (u16, u8) {
    let mut n = 0;
    loop {
        let ch = read_char();
        if ch < b'0' || ch > b'9' {
            return (n, ch);
        }
        let digit = (ch - b'0') as u16;

        n *= 10;
        n += digit;
    }
}
