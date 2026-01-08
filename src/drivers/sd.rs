use alloc::vec::Vec;
use core::{
    marker::Copy,
    ops::{Bound, RangeBounds},
    result, slice,
};

pub use proc_macros::sector_layout;

unsafe extern "C" {
    fn sd_init() -> i32;
    fn sd_readblock(lba: u32, buf: *mut u8, num: u32) -> i32;
    fn sd_writeblock(buf: *const u8, lba: u32, num: u32) -> i32;
}

pub const SECTOR_SIZE: usize = 512;

pub fn init() {
    let code = unsafe { sd_init() };
    if code != 0 {
        panic!("SD initialisation error");
    }
}

pub struct SectorBuf {
    buf: Vec<Sector>, // Used instead of boxed slice to allow buffer size to change.
    start_sector: u32,
}

#[derive(Copy, Clone)]
#[repr(C, align(32))]
struct Sector {
    buf: [u8; SECTOR_SIZE],
}

#[derive(Debug, PartialEq)]
pub enum Error {
    Read,
    Write,
}
pub type Result = result::Result<(), Error>;

impl SectorBuf {
    pub fn new(start_sector: usize, sectors: usize) -> Self {
        Self {
            buf: vec![Sector::zeroed(); sectors],
            start_sector: start_sector as u32,
        }
    }

    pub fn read(&mut self) -> Result {
        let code = unsafe {
            sd_readblock(
                self.start_sector,
                self.mut_inner_buf_ptr(),
                self.sectors() as u32,
            )
        };
        if code == 0 { Err(Error::Read) } else { Ok(()) }
    }

    pub fn write(&mut self) -> Result {
        let code = unsafe {
            sd_writeblock(
                self.mut_inner_buf_ptr(),
                self.start_sector,
                self.sectors() as u32,
            )
        };
        if code == 0 { Err(Error::Write) } else { Ok(()) }
    }

    pub fn resize(&mut self, sectors: usize) {
        self.buf.resize(sectors, Sector::zeroed());
    }

    pub fn sectors(&self) -> usize {
        self.buf.len()
    }

    pub fn as_buf(&self, sector_range: impl RangeBounds<usize>) -> &[u8] {
        let (start_sector, end_sector) = self.sector_range_bounds(sector_range);
        let bytes = (end_sector - start_sector) * SECTOR_SIZE;

        unsafe {
            let ptr = self.inner_buf_ptr().add(start_sector * SECTOR_SIZE);
            slice::from_raw_parts(ptr, bytes)
        }
    }

    pub fn as_mut_buf(&mut self, sector_range: impl RangeBounds<usize>) -> &mut [u8] {
        let (start_sector, end_sector) = self.sector_range_bounds(sector_range);
        let bytes = (end_sector - start_sector) * SECTOR_SIZE;

        unsafe {
            let ptr = self.mut_inner_buf_ptr().add(start_sector * SECTOR_SIZE);
            slice::from_raw_parts_mut(ptr, bytes)
        }
    }

    pub fn clear(&mut self) {
        for sector in self.buf.iter_mut() {
            *sector = Sector::zeroed();
        }
    }

    fn inner_buf_ptr(&self) -> *const u8 {
        self.buf.as_ptr() as *const u8
    }

    fn mut_inner_buf_ptr(&mut self) -> *mut u8 {
        self.buf.as_mut_ptr() as *mut u8
    }

    fn sector_range_bounds(&self, sector_range: impl RangeBounds<usize>) -> (usize, usize) {
        let start_sector = match sector_range.start_bound() {
            Bound::Included(&x) => x,
            Bound::Excluded(&x) => x + 1,
            Bound::Unbounded => 0,
        };
        let end_sector = match sector_range.end_bound() {
            Bound::Included(&x) => x + 1,
            Bound::Excluded(&x) => x,
            Bound::Unbounded => self.sectors(),
        };

        assert!(end_sector <= self.sectors());

        (start_sector, end_sector)
    }
}

impl Sector {
    fn zeroed() -> Self {
        Self {
            buf: [0; SECTOR_SIZE],
        }
    }
}
