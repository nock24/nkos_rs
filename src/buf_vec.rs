#![allow(dead_code)]

use core::{
    mem::MaybeUninit,
    slice,
    fmt,
    marker::Copy,
    ops::{Deref, DerefMut},
    result,
};

pub struct BufVec<T, const N: usize> {
    buf: [MaybeUninit<T>; N],
    len: usize,
}

#[derive(Debug)]
pub enum Error {
    Full,
    Empty,
}

pub type Result<T> = result::Result<T, Error>;

impl<T, const N: usize> BufVec<T, N>
where
    T: Copy,
{
    pub const fn new() -> Self {
        Self {
            buf: [MaybeUninit::uninit(); N],
            len: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn get(&self, idx: usize) -> Option<&T> {
        if idx < self.len {
            unsafe { Some(self.buf[idx].assume_init_ref()) }
        } else {
            None
        }
    }

    pub const fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if idx < self.len {
            unsafe { Some(self.buf[idx].assume_init_mut()) }
        } else {
            None
        }
    }

    pub fn push(&mut self, item: T) -> Result<()> {
        if self.len >= N {
            return Err(Error::Full);
        }

        self.len += 1;
        self.buf[self.len - 1].write(item);
        Ok(())
    }

    pub fn pop(&mut self) -> Result<T> {
        if self.len == 0 {
            return Err(Error::Empty);
        }

        self.len -= 1;
        unsafe { Ok(self.buf[self.len].assume_init()) }
    }

    pub fn insert(&mut self, item: T, idx: usize) -> Result<()> {
        if self.len >= N {
            return Err(Error::Full);
        }

        for i in (idx+1 ..= self.len).rev() {
            self.buf[i].write(*self.get(i-1).unwrap());
        }
        self.len += 1;
        self.buf[idx].write(item);
        Ok(())
    }

    pub fn remove(&mut self, idx: usize) -> Option<T> {
        if idx >= self.len {
            return None;
        }

        let item = unsafe { self.buf[idx].assume_init() };
        for i in idx .. self.len-1 {
            self.buf[i].write(*self.get(i+1).unwrap());
        }
        self.len -= 1;
        Some(item)
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(
            self.buf.as_ptr() as *const T,
            self.len,
        ) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(
            self.buf.as_mut_ptr() as *mut T,
            self.len,
        ) }
    }
}

impl<T, const N: usize> Deref for BufVec<T, N>
where
    T: Copy,
{
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const N: usize> DerefMut for BufVec<T, N>
where
    T: Copy,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T, const N: usize> fmt::Debug for BufVec<T, N>
where
    T: fmt::Debug + Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.as_slice())
            .finish()
    }
}
