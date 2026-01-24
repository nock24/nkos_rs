use alloc::{boxed::Box, vec::Vec};

use super::cmd_parser::{self, Arg, Args};
use crate::{BootSector, drivers::sd, heap, nkvi};

use proc_macros::cmd_try_froms;

pub trait Cmd {
    fn run(&self);
}

pub fn new<'a>(ident: &'a [u8], args: Args<'a>) -> cmd_parser::Result<Box<dyn Cmd + 'a>> {
    cmd_try_froms!(ident, args, Echo, BootCnt, HeapChunks, Nkvi,)
}

pub struct Echo<'a> {
    str: &'a str,
}

impl<'a> Echo<'a> {
    pub const IDENT: &'static [u8] = b"echo";
}

impl<'a> Cmd for Echo<'a> {
    fn run(&self) {
        sprintln!("{}", self.str);
    }
}

impl<'a> TryFrom<Args<'a>> for Echo<'a> {
    type Error = &'static str;

    fn try_from(args: Args<'a>) -> Result<Self, Self::Error> {
        if args.len() != 1 {
            return Err("invalid number of arguments");
        }

        let Arg::StrQuotes(str) = args[0] else {
            return Err("invalid argument type");
        };

        Ok(Self { str })
    }
}

pub struct BootCnt {
    reset: bool,
}

impl BootCnt {
    pub const IDENT: &'static [u8] = b"boot-cnt";
}

impl Cmd for BootCnt {
    fn run(&self) {
        let mut sector_buf = sd::SectorBuf::new(0, 1);

        if let Err(e) = sector_buf.read() {
            assert_eq!(e, sd::Error::Read);
            sprintln!("SD read error");
        }

        if self.reset {
            BootSector::set_boot_cnt(sector_buf.as_mut_buf(..), 0);
            if let Err(e) = sector_buf.write() {
                assert_eq!(e, sd::Error::Write);
                sprintln!("SD write error");
            }
            sprintln!("boot count reset");
        } else {
            let boot_cnt = BootSector::boot_cnt(sector_buf.as_buf(..));
            sprintln!("boot count: {}", boot_cnt);
        }
    }
}

impl<'a> TryFrom<Args<'a>> for BootCnt {
    type Error = &'static str;

    fn try_from(args: Args<'a>) -> Result<Self, Self::Error> {
        let reset = match args.len() {
            0 => false,
            1 => {
                let Arg::Str(str) = args[0] else {
                    return Err("invalid argument type");
                };
                if str != "reset" {
                    return Err("expected 'reset' argument");
                }
                true
            }
            _ => return Err("too many arguments"),
        };

        Ok(Self { reset })
    }
}

struct HeapChunks;

impl<'a> HeapChunks {
    pub const IDENT: &'static [u8] = b"heap-chunks";
}

impl Cmd for HeapChunks {
    fn run(&self) {
        heap::print_chunks();
    }
}

impl<'a> TryFrom<Args<'a>> for HeapChunks {
    type Error = &'static str;

    fn try_from(args: Args<'a>) -> Result<Self, Self::Error> {
        if args.len() > 0 {
            Err("expected no arguments")
        } else {
            Ok(Self)
        }
    }
}

struct Nkvi;

impl Nkvi {
    pub const IDENT: &'static [u8] = b"nkvi";
}

impl Cmd for Nkvi {
    fn run(&self) {
        nkvi::run();
    }
}

impl<'a> TryFrom<Args<'a>> for Nkvi {
    type Error = &'static str;

    fn try_from(args: Args<'a>) -> Result<Self, Self::Error> {
        if args.len() > 0 {
            Err("expected no arguments")
        } else {
            Ok(Self)
        }
    }
}
