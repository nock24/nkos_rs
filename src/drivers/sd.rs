use core::{
    result,
    marker::{Copy, PhantomData},
    ops::{Bound, RangeBounds},
    slice,
};
use alloc::boxed::Box;

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

//pub struct SectorBuf<const BYTES: usize> {
pub struct LayoutBuf<L: SectorLayout, const SECTORS: usize> {
    buf: Box<[Sector; SECTORS]>,
    start_sector: u32,
    _phantom: PhantomData<L>,
}

pub struct DynSectorBuf {
    buf: Box<[Sector]>,
    start_sector: u32,
}

#[derive(Copy, Clone)]
#[repr(C, align(32))]
struct Sector {
    buf: [u8; SECTOR_SIZE]
}

#[derive(Debug, PartialEq)]
pub enum Error {
    Read,
    Write,
}
pub type Result = result::Result<(), Error>;

macro_rules! layout_buf_ty {
    ($L:ty) => {
        $crate::drivers::sd::LayoutBuf<$L, { <$L>::SECTORS }>
    };
}
pub(crate) use layout_buf_ty;

macro_rules! layout_buf {
    ($L:ty, $start_sector:expr) => {{
        $crate::drivers::sd::LayoutBuf::<$L, { <$L>::SECTORS }>::new($start_sector)
    }};
}
pub(crate) use layout_buf;

pub unsafe trait SectorLayout {
    const SECTORS: usize;
    const BYTES: usize = Self::SECTORS * SECTOR_SIZE;
}

/*
macro_rules! sector_layout {
    {
        $vis:vis $name:ident {
            $($field:ident : $T:ty),+ $(,)?
        }
    } => {
        #[repr(C)]
        $vis struct $name {
            $(pub $field: $T),+
        }

        const_assert_item!(core::mem::align_of::<$name>() <= 32);

        const_block! {
            const SIZE: usize = core::mem::size_of::<$name>();
            const SECTOR_SIZE: usize = $crate::drivers::sd::SECTOR_SIZE;
            const SECTORS: usize = SIZE.div_ceil(SECTOR_SIZE);

            impl $name {
                const SECTORS: usize = SECTORS;
            }

            unsafe impl $crate::drivers::sd::SectorLayout for $name {
                const SECTORS: usize = SECTORS;
            }
        }
    }
}
pub(crate) use sector_layout;
*/

//impl<const BYTES: usize> SectorBuf<BYTES> {
impl<L: SectorLayout, const SECTORS: usize> LayoutBuf<L, SECTORS> {
    pub fn new(start_sector: usize) -> Self {
        const_assert!(SECTORS == L::SECTORS);

        Self {
            buf: Box::new([Sector::new(); SECTORS]),
            start_sector: start_sector as u32,
            _phantom: PhantomData,
        }
    }

    pub fn read(&mut self) -> Result {
        let code = unsafe {
            sd_readblock(self.start_sector, self.mut_buf_ptr(), SECTORS as u32)
        };
        if code == 0 {
            Err(Error::Read)
        } else {
            Ok(())
        }
    }

    pub fn write(&mut self) -> Result {
        let code = unsafe {
            sd_writeblock(self.mut_buf_ptr(), self.start_sector, SECTORS as u32)
        };
        if code == 0 {
            Err(Error::Write)
        } else {
            Ok(())
        }
    }

    pub fn as_layout(&self) -> &L {
        unsafe {
            let ptr = self.buf_ptr() as *const L;
            &*ptr
        }
    }

    pub fn as_mut_layout(&mut self) -> &mut L {
        unsafe {
            let ptr = self.mut_buf_ptr() as *mut L;
            &mut *ptr
        }
    }

    pub fn clear(&mut self) {
        for sector in self.buf.iter_mut() {
            sector.buf = [0; SECTOR_SIZE];
        }
    }

    fn buf_ptr(&self) -> *const u8 {
        self.buf.as_ptr() as *const u8
    }

    fn mut_buf_ptr(&mut self) -> *mut u8 {
        self.buf.as_mut_ptr() as * mut u8
    }
}

impl DynSectorBuf {
    pub fn new(start_sector: usize, sectors: usize) -> Self {
        Self {
            buf: vec![Sector::new(); sectors].into_boxed_slice(),
            start_sector: start_sector as u32,
        }
    }

    pub fn read(&mut self) -> Result {
        let code = unsafe {
            sd_readblock(self.start_sector, self.mut_buf_ptr(), self.sectors() as u32)
        };
        if code == 0 {
            Err(Error::Read)
        } else {
            Ok(())
        }
    }

    pub fn write(&mut self) -> Result {
        let code = unsafe {
            sd_writeblock(self.mut_buf_ptr(), self.start_sector, self.sectors() as u32)
        };
        if code == 0 {
            Err(Error::Write)
        } else {
            Ok(())
        }
    }

    pub fn sectors(&self) -> usize {
        self.buf.len()
    }

    /// `sector_range` is relative to this buffer's sectors not the lba of the sectors.
    pub fn as_layout<L: SectorLayout>(
        &self,
        sector_range: impl RangeBounds<usize>,
    ) -> &L
    {
        let (start_sector, end_sector) = self.sector_range_bounds(sector_range);
        assert_eq!(L::SECTORS, end_sector - start_sector);
        
        unsafe {
            let ptr = self.buf_ptr().add(start_sector * SECTOR_SIZE) as *const L;
            &*ptr
        }
    }

    pub fn as_mut_layout<L: SectorLayout>(
        &mut self,
        sector_range: impl RangeBounds<usize>,
    ) -> &mut L
    {
        let (start_sector, end_sector) = self.sector_range_bounds(sector_range);
        assert_eq!(L::SECTORS, end_sector - start_sector);

        unsafe {
            let ptr = self.mut_buf_ptr().add(start_sector * SECTOR_SIZE) as *mut L;
            &mut *ptr
        }
    }

    pub fn as_dyn_buf(&self, sector_range: impl RangeBounds<usize>) -> &[u8] {
        let (start_sector, end_sector) = self.sector_range_bounds(sector_range);
        let bytes = (end_sector - start_sector) * SECTOR_SIZE;

        unsafe {
            let ptr = self.buf_ptr().add(start_sector * SECTOR_SIZE);
            slice::from_raw_parts(ptr, bytes)
        }
    }

    pub fn as_mut_dyn_buf(&mut self, sector_range: impl RangeBounds<usize>) -> &mut [u8] {
        let (start_sector, end_sector) = self.sector_range_bounds(sector_range);
        let bytes = (end_sector - start_sector) * SECTOR_SIZE;

        unsafe {
            let ptr = self.mut_buf_ptr().add(start_sector * SECTOR_SIZE);
            slice::from_raw_parts_mut(ptr, bytes)
        }
    }

    pub fn clear(&mut self) {
        for sector in self.buf.iter_mut() {
            sector.buf = [0; SECTOR_SIZE];
        }
    }

    fn buf_ptr(&self) -> *const u8 {
        self.buf.as_ptr() as *const u8
    }

    fn mut_buf_ptr(&mut self) -> *mut u8 {
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
    fn new() -> Self {
        Self { buf: [0; SECTOR_SIZE] }
    }
}
