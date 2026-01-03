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
}

#[repr(C, align(32))]
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

pub trait SectorLayout<const SECTORS: usize> {}

macro_rules! sector_layout {
    {
        $vis:vis $name:ident {
            $($f:ident : $t:ty),+ $(,)?
        }
    } => {
        #[repr(C)]
        $vis struct $name {
            $(pub $f: $t),+
        }

        const _: () = {
            assert!(core::mem::align_of::<$name>() <= 32);

            const SIZE: usize = core::mem::size_of::<$name>();
            const SECTOR_SIZE: usize = $crate::drivers::sd::SECTOR_SIZE;
            const SECTORS: usize = SIZE.div_ceil(SECTOR_SIZE);

            impl $crate::drivers::sd::SectorLayout<SECTORS> for $name {}
        };
    }
}
pub(crate) use sector_layout;

impl<const N: usize> AlignedBuf<N> {
    fn new() -> Self {
        Self {
            buf: [0; N],
        }
    }

    fn as_ptr(&self) -> *const u8 {
        self.buf.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut u8 {
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
        const { assert!(BYTES % SECTOR_SIZE == 0); }

        Self {
            buf: Box::new(AlignedBuf::new()),
            start_sector: start_sector as u32,
        }
    }

    const SECTORS: usize = BYTES / SECTOR_SIZE;

    pub fn read(&mut self) -> Result {
        let code = unsafe {
            sd_readblock(self.start_sector, self.buf.as_mut_ptr(), Self::SECTORS as u32)
        };
        if code == 0 {
            Err(Error::Read)
        } else {
            Ok(())
        }
    }

    pub fn write(&mut self) -> Result {
        let code = unsafe {
            sd_writeblock(self.buf.as_mut_ptr(), self.start_sector, Self::SECTORS as u32)
        };
        if code == 0 {
            Err(Error::Write)
        } else {
            Ok(())
        }
    }

    pub fn as_layout<T, const N: usize>(&self) -> &T
    where
        T: SectorLayout<N>,
    {
        const { assert!(N == Self::SECTORS); }
        
        unsafe {
            let ptr = self.buf.as_ptr() as *const T;
            &*ptr
        }
    }

    pub fn as_mut_layout<T, const N: usize>(&mut self) -> &mut T
    where
        T: SectorLayout<N>,
    {
        const { assert!(N == Self::SECTORS); }

        unsafe {
            let ptr = self.buf.as_mut_ptr() as *mut T;
            &mut *ptr
        }
    }

    /*
    pub fn get_val<T>(&self, offset: usize) -> Option<&T> {
        if offset >= BYTES {
            None
        } else {
            unsafe {
                let ptr = self.buf.as_ptr().add(offset) as *const T;
                if ptr.is_aligned() {
                    Some(&*ptr)
                } else {
                    None
                }
            }
        }
    }

    pub fn get_mut_val<T>(&mut self, offset: usize) -> Option<&mut T> {
        if offset >= BYTES {
            None
        } else {
            unsafe {
                let ptr = self.buf.as_mut_ptr().add(offset) as *mut T;
                if ptr.is_aligned() {
                    Some(&mut *ptr)
                } else {
                    None
                }
            }
        }
    }
    */
}
