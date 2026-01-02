use core::result;
use alloc::boxed::Box;

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

pub const SECTOR_SIZE: usize = 512;

pub struct SectorBuf<const BYTES: usize> {
    buf: Box<AlignedBuf<BYTES>>,
    start_sector: u32,
    sectors: u32,
}

#[repr(align(32))]
struct AlignedBuf<const N: usize> {
    buf: [u8; N],
}

macro_rules! sector_buf {
    ($start_sector:expr, $sectors:expr) => {{
        const BYTES: usize = $sectors * $crate::drivers::sd::SECTOR_SIZE;
        $crate::drivers::sd::SectorBuf::<BYTES>::new($start_sector)
    }};
}
pub(crate) use sector_buf;

impl<const N: usize> AlignedBuf<N> {
    pub fn new() -> Self {
        Self {
            buf: [0; N],
        }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.buf.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.buf.as_mut_ptr()
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    Read,
    Write,
}

pub type Result = result::Result<(), Error>;

impl<const BYTES: usize> SectorBuf<BYTES> {
    pub fn new(start_sector: usize) -> Self {
        assert!(BYTES % SECTOR_SIZE == 0);

        Self {
            buf: Box::new(AlignedBuf::new()),
            start_sector: start_sector as u32,
            sectors: (BYTES / SECTOR_SIZE) as u32,
        }
    }

    pub fn read(&mut self) -> Result {
        let code = unsafe {
            sd_readblock(self.start_sector, self.buf.as_mut_ptr(), self.sectors)
        };
        if code == 0 {
            Err(Error::Read)
        } else {
            Ok(())
        }
    }

    pub fn write(&mut self) -> Result {
        let code = unsafe {
            sd_writeblock(self.buf.as_mut_ptr(), self.start_sector, self.sectors)
        };
        if code == 0 {
            Err(Error::Write)
        } else {
            Ok(())
        }
    }

    pub fn get_val<T>(&self, offset: usize) -> Option<&T> {
        if offset >= BYTES {
            None
        } else {
            unsafe {
                let ptr = self.buf.as_ptr().add(offset) as *const T;
                ptr.as_ref()
            }
        }
    }

    pub fn get_mut_val<T>(&mut self, offset: usize) -> Option<&mut T> {
        if offset >= BYTES {
            None
        } else {
            unsafe {
                let ptr = self.buf.as_mut_ptr().add(offset) as *mut T;
                ptr.as_mut()
            }
        }
    }
}
