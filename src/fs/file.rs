use core::{
    result,
    str,
};
use alloc::boxed::Box;

use crate::drivers::sd;

pub trait File {
    fn read(&mut self) -> Result<()>;
    fn write(&mut self) -> Result<()>;
}

#[repr(u8)]
enum FileType {
    Text = 0,
}

pub struct TextFile {
    sector_buf: sd::SectorBuf,
    str: Option<Box<[u8]>>,
}

sd::sector_layout! {
    pub TextLayout {
        type_code: u8,
        str_len: u8,
        str: [u8; str_len],
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    FileWrongType,
    Sd(sd::Error),
}
pub type Result<T> = result::Result<T, Error>;

impl TextFile {
    pub fn new(start_sector: usize) -> Result<Self> {
        let mut sector_buf = sd::SectorBuf::new(start_sector, TextLayout::HEADER_SECTORS);
        sector_buf.read()?;
        let sectors = TextLayout::sectors(sector_buf.as_buf(..));
        sector_buf.resize(sectors);

        Ok(Self {
            sector_buf,
            str: None,
        })
    }

    pub fn str<'a>(&'a self) -> Option<&'a str> {
        let Some(str) = &self.str else {
            return None;
        };
        str::from_utf8(str.as_ref()).ok()
    }

    pub fn mut_str<'a>(&'a mut self) -> Option<&'a mut str> {
        let Some(str) = &mut self.str else {
            return None;
        };
        str::from_utf8_mut(str.as_mut()).ok()
    }
}

impl File for TextFile {
    fn read(&mut self) -> Result<()> {
        self.sector_buf.read()?;

        let type_code = TextLayout::type_code(self.sector_buf.as_buf(..));
        if type_code != FileType::Text as u8 {
            self.sector_buf.clear();
            Err(Error::FileWrongType)
        } else {
            self.str = Some(TextLayout::str_boxed(self.sector_buf.as_buf(..)));
            Ok(())
        }
    }

    fn write(&mut self) -> Result<()> {
        if let Some(str) = &self.str {
            TextLayout::str_write(self.sector_buf.as_mut_buf(..), str.as_ref());
        }
        self.sector_buf.write()?;
        Ok(())
    }
}

impl From<sd::Error> for Error {
    fn from(err: sd::Error) -> Self {
        Self::Sd(err)
    }
}
