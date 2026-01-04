use core::{
    result,
    slice,
    str,
};

use crate::drivers::sd;

pub trait File {
    fn read(&mut self) -> Result;
    fn write(&mut self) -> Result;
}

#[repr(u8)]
enum FileType {
    Text = 0,
}

pub struct TextFile {
    sector_buf: sd::sector_buf_ty!(TextLayout::SECTORS),
}

sd::sector_layout! {
    pub TextLayout {
        type_code: u8,
        str_len: u8,
        str_buf: [u8; 20],
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    FileWrongType,
    Sd(sd::Error),
}
pub type Result = result::Result<(), Error>;

impl TextFile {
    pub fn new(sector: usize) -> Self {
        Self {
            sector_buf: sd::sector_buf!(sector, TextLayout::SECTORS),
        }
    }

    pub fn str<'a>(&'a self) -> Option<&'a str> {
        let layout: &TextLayout = self.sector_buf.as_layout(..);
        let str = unsafe { slice::from_raw_parts(
            layout.str_buf.as_ptr(),
            layout.str_len as usize,
        ) };
        str::from_utf8(str).ok()
    }

    pub fn mut_str<'a>(&'a mut self) -> Option<&'a mut str> {
        let layout: &mut TextLayout = self.sector_buf.as_mut_layout();
        let str = unsafe { slice::from_raw_parts_mut(
            layout.str_buf.as_mut_ptr(),
            layout.str_len as usize,
        ) };
        str::from_utf8_mut(str).ok()
    }

    pub fn set_str_len(&mut self, len: u8) {
        let layout: &mut TextLayout = self.sector_buf.as_mut_layout();
        layout.str_len = len;
    }
}

impl File for TextFile {
    fn read(&mut self) -> Result {
        self.sector_buf.read()?;

        let layout: &TextLayout = self.sector_buf.as_layout();
        if layout.type_code != FileType::Text as u8 {
            self.sector_buf.clear();
            Err(Error::FileWrongType)
        } else {
            Ok(())
        }
    }

    fn write(&mut self) -> Result {
        self.sector_buf.write()?;
        Ok(())
    }
}

impl From<sd::Error> for Error {
    fn from(err: sd::Error) -> Self {
        Self::Sd(err)
    }
}
