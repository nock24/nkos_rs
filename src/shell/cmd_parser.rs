use alloc::{boxed::Box, vec::Vec};
use core::result;

use super::cmd::{self, Cmd};

pub type Result<T> = result::Result<T, &'static str>;

pub fn parse<'a>(str: &'a [u8]) -> Result<Box<dyn Cmd + 'a>> {
    let mut parser = Parser::new(str);
    parser.parse()
}

pub enum Arg<'a> {
    Str(&'a str),
    StrQuotes(&'a str),
    Int(u32),
}

pub type Args<'a> = Vec<Arg<'a>>;

struct Parser<'a> {
    str: &'a [u8],
    idx: usize,
}

impl<'a> Parser<'a> {
    fn new(str: &'a [u8]) -> Self {
        Self { str, idx: 0 }
    }

    fn parse(&mut self) -> Result<Box<dyn Cmd + 'a>> {
        _ = self.skip_whitespace();

        let ident_start = self.idx;
        if self.skip_to_whitespace().is_err() {
            self.idx = self.str.len();
        }
        let ident = &self.str[ident_start..self.idx];
        if ident.len() == 0 {
            return Err("");
        }
        let args = self.parse_args()?;

        cmd::new(ident, args)
    }

    fn parse_args(&mut self) -> Result<Args<'a>> {
        if self.idx == self.str.len() {
            return Ok(Args::new());
        }

        let mut args = Args::new();
        while self.skip_whitespace().is_ok() {
            let start = self.idx;

            args.push(match self.str[start] {
                b'\"' => self.parse_str_quotes_arg()?,
                b'0'..b'9' => todo!(),
                _ => self.parse_str_arg()?,
            });
        }
        Ok(args)
    }

    fn parse_str_arg(&mut self) -> Result<Arg<'a>> {
        let start = self.idx;
        if self.skip_to_whitespace().is_err() {
            self.idx = self.str.len();
        }

        let str = str::from_utf8(&self.str[start..self.idx]).map_err(|_| "failed to parse utf8")?;
        Ok(Arg::Str(str))
    }

    fn parse_str_quotes_arg(&mut self) -> Result<Arg<'a>> {
        if self.str[self.idx] != b'\"' {
            return Err("expected \"");
        }
        self.idx += 1;
        let start = self.idx;
        self.skip_to(|&c| c == b'\"').map_err(|_| "expected \"")?;
        let end = self.idx;
        self.idx += 1;

        let str = str::from_utf8(&self.str[start..end]).map_err(|_| "failed to parse utf8")?;
        Ok(Arg::StrQuotes(str))
    }

    fn skip_to<P>(&mut self, pred: P) -> Result<()>
    where
        P: FnMut(&u8) -> bool,
    {
        if let Some(i) = self.str.iter().skip(self.idx).position(pred) {
            self.idx += i;
            Ok(())
        } else {
            Err("failed to skip")
        }
    }

    fn skip_whitespace(&mut self) -> Result<()> {
        self.skip_to(|&c| c != b' ')
    }

    fn skip_to_whitespace(&mut self) -> Result<()> {
        self.skip_to(|&c| c == b' ')
    }
}
