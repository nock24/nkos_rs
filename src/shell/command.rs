use core::result;
use alloc::{vec::Vec, boxed::Box};

use crate::drivers::{
    serial,
    sd,
};

pub trait Command<'a> {
    fn run(&self);
    fn parse(args_str: &'a [u8]) -> Result<'a> where Self: Sized;
    fn ident() -> &'static [u8] where Self: Sized;
}

pub type Result<'a> = result::Result<Box<dyn Command<'a> + 'a>, &'static str>;

pub fn parse<'a>(str: &'a Vec<u8>) -> Result<'a> {
    let Some(ident_start) = skip_whitespace(str, 0) else {
        return Err("");
    };
    let ident_end = skip_chars(str, ident_start).unwrap_or(str.len());

    let args_str = if let Some(args_start) = skip_whitespace(str, ident_end) {
        &str[args_start..]
    } else {
        &[]
    };
    
    let ident = &str[ident_start..ident_end];
    parse_from_ident(ident, args_str)
} 

fn parse_from_ident<'a>(ident: &'a [u8], args_str: &'a [u8]) -> Result<'a> {
    if ident == Echo::ident() {
        Echo::parse(args_str)
    } else if ident == BootCnt::ident() {
        BootCnt::parse(args_str)
    } else {
        Err("Invalid command.")
    }
}

struct Echo<'a> {
    str: &'a str,
}

impl<'a> Command<'a> for Echo<'a> {
    fn run(&self) {
        serial::println!("{}", self.str);
    }

    fn parse(args_str: &'a [u8]) -> Result<'a>
    where 
        Self: Sized,
    {
        if args_str.len() == 0 {
            return Err("Provide string argument.")
        };
        if args_str[0] != b'\"' {
            return Err("Expected \".");
        }

        let Some(end_idx) = skip(args_str, 1, |&c| c == b'\"') else {
            return Err("Expected \".");
        };

        if let Some(_) = skip_whitespace(args_str, end_idx + 1) {
            return Err("Only provide string argument.");
        }

        let str = &args_str[1..end_idx];
        Ok(Box::new(Self {
            str: unsafe { str::from_utf8_unchecked(str) },
        }))
    }

    fn ident() -> &'static [u8]
    where 
        Self: Sized,
    {
        b"echo"
    }
}

struct BootCnt {
    reset: bool,
}

impl<'a> Command<'a> for BootCnt {
    fn run(&self) {
        let mut buf = sd::sector_buf!(0, 1);

        if let Err(e) = buf.read() {
            assert_eq!(e, sd::Error::Read);
            serial::println!("SD read error.");
        }

        let boot_cnt: &mut usize = buf.get_mut_val(0).unwrap();

        if self.reset {
            *boot_cnt = 0;
            if let Err(e) = buf.write() {
                assert_eq!(e, sd::Error::Write);
                serial::println!("SD write error.");
            }
            serial::println!("Boot count reset.");
        } else {
            serial::println!("Boot count: {}", *boot_cnt);
        }
    }

    fn parse(args_str: &'a [u8]) -> Result<'a>
    where
        Self: Sized,
    {
        if args_str.len() == 0 {
            Ok(Box::new(Self {
                reset: false,
            }))
        } else {
            let end_idx = skip_chars(args_str, 0).unwrap_or(args_str.len());
            match &args_str[..end_idx] {
                b"reset" => Ok(Box::new(Self {
                    reset: true,
                })),
                _ => Err("Invalid argument."),
            }
        }
    }

    fn ident() -> &'static [u8]
    where
        Self: Sized,
    {
        b"bootcnt"
    }
}

fn skip<P>(str: &[u8], start_idx: usize, pred: P) -> Option<usize>
where
    P: FnMut(&u8) -> bool,
{
    if let Some(x) = str.iter()
        .skip(start_idx)
        .position(pred)
    {
        Some(start_idx + x)
    } else {
        None
    }
}

fn skip_whitespace(str: &[u8], start_idx: usize) -> Option<usize> {
    skip(str, start_idx, |&c| c != b' ')
}

fn skip_chars(str: &[u8], start_idx: usize) -> Option<usize> {
    skip(str, start_idx, |&c| c == b' ')
}
