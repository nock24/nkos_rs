use alloc::boxed::Box;
use core::result;

use crate::{BootSector, drivers::sd, heap, nkvi};

pub trait Command<'a> {
    fn run(&self);
}

pub type Result<'a> = result::Result<Box<dyn Command<'a> + 'a>, &'static str>;

pub fn parse<'a>(str: &'a [u8]) -> Result<'a> {
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

type ParseFn<'a> = fn(&'a [u8]) -> Result<'a>;

fn parse_from_ident<'a>(ident: &'a [u8], args_str: &'a [u8]) -> Result<'a> {
    let parse_fn: ParseFn<'a> = match ident {
        Echo::IDENT => Echo::parse,
        BootCnt::IDENT => BootCnt::parse,
        HeapChunks::IDENT => HeapChunks::parse,
        Nkvi::IDENT => Nkvi::parse,
        _ => return Err("Invalid command."),
    };

    parse_fn(args_str)
}

struct Echo<'a> {
    str: &'a str,
}

impl<'a> Echo<'a> {
    const IDENT: &'static [u8] = b"echo";

    fn parse(args_str: &'a [u8]) -> Result<'a> {
        if args_str.len() == 0 {
            return Err("Provide string argument.");
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
}

impl<'a> Command<'a> for Echo<'a> {
    fn run(&self) {
        sprintln!("{}", self.str);
    }
}

struct BootCnt {
    reset: bool,
}

impl<'a> BootCnt {
    const IDENT: &'static [u8] = b"boot-cnt";

    fn parse(args_str: &'a [u8]) -> Result<'a> {
        if args_str.len() == 0 {
            Ok(Box::new(Self { reset: false }))
        } else {
            let end_idx = skip_chars(args_str, 0).unwrap_or(args_str.len());
            match &args_str[..end_idx] {
                b"reset" => Ok(Box::new(Self { reset: true })),
                _ => Err("Invalid argument."),
            }
        }
    }
}

impl<'a> Command<'a> for BootCnt {
    fn run(&self) {
        let mut sector_buf = sd::SectorBuf::new(0, 1);

        if let Err(e) = sector_buf.read() {
            assert_eq!(e, sd::Error::Read);
            sprintln!("SD read error.");
        }

        if self.reset {
            BootSector::set_boot_cnt(sector_buf.as_mut_buf(..), 0);
            if let Err(e) = sector_buf.write() {
                assert_eq!(e, sd::Error::Write);
                sprintln!("SD write error.");
            }
            sprintln!("Boot count reset.");
        } else {
            let boot_cnt = BootSector::boot_cnt(sector_buf.as_buf(..));
            sprintln!("Boot count: {}", boot_cnt);
        }
    }
}

struct HeapChunks;

impl<'a> HeapChunks {
    const IDENT: &'static [u8] = b"heap-chunks";

    fn parse(args_str: &'a [u8]) -> Result<'a> {
        if args_str.len() != 0 {
            Err("Command takes no arguments.")
        } else {
            Ok(Box::new(Self))
        }
    }
}

impl<'a> Command<'a> for HeapChunks {
    fn run(&self) {
        heap::print_chunks();
    }
}

struct Nkvi;

impl<'a> Nkvi {
    const IDENT: &'static [u8] = b"nkvi";

    fn parse(args_str: &'a [u8]) -> Result<'a> {
        if args_str.len() != 0 {
            Err("Command takes no arguments.")
        } else {
            Ok(Box::new(Self))
        }
    }
}

impl<'a> Command<'a> for Nkvi {
    fn run(&self) {
        nkvi::run();
    }
}

fn skip<P>(str: &[u8], start_idx: usize, pred: P) -> Option<usize>
where
    P: FnMut(&u8) -> bool,
{
    if let Some(x) = str.iter().skip(start_idx).position(pred) {
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
