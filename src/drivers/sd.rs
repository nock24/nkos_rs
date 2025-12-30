use alloc::vec::Vec;

unsafe extern "C" {
    fn sd_init() -> i32;
    fn sd_readblock(lba: u32, buf: *mut u8, num: u32) -> i32;
    fn sd_writeblock(buf: *const u8, lba: u32, num: u32) -> i32;
}

pub fn init() {
    let code = unsafe { sd_init() };
    if code != 0 {
        panic!("SD initialisation error");
    }
}

#[derive(Debug)]
pub enum Error {
    Read,
    Write,
}

pub const SECTOR_SIZE: usize = 512;

pub struct SectorBuf {
    buf: Vec<u8>,
    size: u32,
    start_sector: u32,
}

impl SectorBuf {
    pub fn new(start_sector: usize, size: usize) -> Self {
        Self {
            buf: vec![0; size * SECTOR_SIZE],
            size: size as u32,
            start_sector: start_sector as u32,
        }
    }

    pub fn read(&mut self) -> Result<(), Error> {
        let buf_ptr = self.buf.as_mut_ptr();
        let code = unsafe { sd_readblock(self.start_sector, buf_ptr, self.size) };

        if code == 0 {
            Err(Error::Read)
        } else {
            Ok(())
        }
    }

    pub fn write(&mut self) -> Result<(), Error> {
        let buf_ptr = self.buf.as_mut_ptr();
        let code = unsafe { sd_writeblock(buf_ptr, self.start_sector, self.size) };

        if code == 0 {
            Err(Error::Write)
        } else {
            Ok(())
        }
    }

    pub fn get_val<T>(&self, offset: usize) -> Option<&T> {
        if offset >= self.buf.len() {
            None
        } else {
            let ptr = &self.buf[offset] as *const u8 as *const T;
            unsafe { ptr.as_ref() }
        }
    }

    pub fn get_mut_val<T>(&mut self, offset: usize) -> Option<&mut T> {
        if offset >= self.buf.len() {
            None
        } else {
            let ptr = &mut self.buf[offset] as *mut u8 as *mut T;
            unsafe { ptr.as_mut() }
        }
    }
}
