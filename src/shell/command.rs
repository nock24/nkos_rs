use alloc::vec::Vec;

use crate::drivers::serial;

pub trait Command<'a> {
    const IDENT: &'static [u8];

    fn parse(args: &'a [u8]) -> Result<impl Command<'a>, &'static str>;
    fn run(self);
}

pub fn parse_cmd<'a>(str: &'a Vec<u8>) -> Result<impl Command<'a>, &'static str> {
    let Some(ident_start) = skip_whitespace(str, 0) else {
        return Err("");
    };
    let ident_end = skip_chars(str, ident_start).unwrap_or(str.len());

    let args_str = if let Some(args_start) = skip_whitespace(str, ident_end) {
        &str[args_start..]
    } else {
        &[]
    };

    match &str[ident_start..ident_end] {
        Echo::IDENT => Echo::parse(args_str),
        _ => Err("Invalid command."),
    }
} 

struct Echo<'a> {
    str: &'a str,
}

impl<'a> Command<'a> for Echo<'a> {
    const IDENT: &'static [u8] = b"echo";

    fn parse(args_str: &'a [u8]) -> Result<impl Command<'a>, &'static str> {
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
        Ok(Self {
            str: unsafe { str::from_utf8_unchecked(str) },
        })
    }

    fn run(self) {
        serial::println!("{}", self.str);
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
