#![allow(dead_code)]

use core::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
    marker::Copy,
};

use crate::{
    buf_vec::BufVec,
    drivers::serial,
    linker_ptrs::{heap_size, heap_start},
};

#[global_allocator]
static ALLOCATOR: Allocator<64> = Allocator::new();

#[derive(Clone, Copy)]
struct Chunk {
    pub offset: usize,
    pub size: usize,
    pub free: bool,
}

struct Heap<const N: usize> {
    buf: *mut u8,
    cur_size: usize,
    pub chunks: BufVec<Chunk, N>,
}

impl<const N: usize> Heap<N> {
    pub const fn new() -> Self {
        unsafe { Self {
            buf: heap_start(),
            cur_size: 0,
            chunks: BufVec::new(),
        } }
    }

    fn max_size(&self) -> usize {
        unsafe { heap_size() }
    }

    pub fn add_size(&mut self, n: usize) {
        self.cur_size += n;
        assert!(self.cur_size <= self.max_size(), "Heap out of memory.");
    }

    pub fn push_chunk(&mut self, chunk: Chunk) {
        self.chunks.push(chunk).expect("Heap out of chunks.");
    }

    pub fn insert_chunk(&mut self, chunk: Chunk, idx: usize) {
        self.chunks.insert(chunk, idx).expect("Heap out of chunks.");
    }

    pub const fn chunk_to_ptr(&self, chunk: &Chunk) -> *mut u8 {
        unsafe { self.buf.add(chunk.offset) }
    }

    pub const fn chunk_from_ptr(&self, ptr: *mut u8, size: usize, free: bool) -> Chunk {
        unsafe { Chunk {
            offset: self.buf.offset_from_unsigned(ptr),
            size,
            free,
        } }
    }

    pub fn align_offset(&self, offset: usize, layout: Layout) -> usize {
        unsafe {
            let unaligned = self.buf.add(offset);
            unaligned.align_offset(layout.align())
        }
    }

    pub fn useable_chunk(&self, chunk: &Chunk, layout: Layout) -> bool {
        let align_offset = self.align_offset(chunk.offset, layout);

        chunk.free && align_offset < chunk.size && layout.size() <= chunk.size - align_offset
    }

    /// The chunk at `idx` is changed to the correct alignment and size, and `free` is set to 
    /// `false` as this function is used when a chunk is being allocated to a user.
    /// The pointer for the chunk is returned.
    /// Extra chunks before and after due to alignment and excess size are added to `self.chunks`.

    pub fn layout_chunk(&mut self, idx: usize, layout: Layout) -> *mut u8 {
        assert!(self.useable_chunk(&self.chunks[idx], layout));
        let mut chunk = self.chunks[idx];
        chunk.free = false;

        let align_offset = self.align_offset(chunk.offset, layout);

        let extra_before = if align_offset > 0 {
            Some(Chunk {
                offset: chunk.offset,
                size: align_offset,
                free: true,
            })
        } else {
            None
        };

        chunk.offset += align_offset;
        chunk.size -= align_offset;

        let extra_size = chunk.size - layout.size();
        let extra_after = if extra_size > 0 {
            chunk.size = layout.size();
            Some(Chunk {
                offset: chunk.offset + chunk.size,
                size: extra_size,
                free: true,
            })
        } else {
            None
        };

        self.chunks[idx] = chunk;
        let ptr = self.chunk_to_ptr(&chunk);

        if let Some(extra_before) = extra_before {
            self.insert_chunk(extra_before, idx);
            if let Some(extra_after) = extra_after {
                self.insert_chunk(extra_after, idx+2);
            }
        } else {
            if let Some(extra_after) = extra_after {
                self.insert_chunk(extra_after, idx+1);
            }
        }

        ptr
    }

    fn defrag(&mut self) {
        if self.chunks.len() <= 1 {
            return;
        }

        let mut i = 0;
        while i < self.chunks.len() - 1 {
            if !(self.chunks[i].free && self.chunks[i+1].free) {
                i += 1;
                continue;
            }

            let other_chunk = self.chunks.remove(i+1).unwrap();
            self.chunks[i].size += other_chunk.size;
        }
    }

    fn debug_chunks(&self, context: &'static str) {
        serial::write("["); serial::write(context); serial::write("]\n");

        for (i, chunk) in self.chunks.iter().enumerate() {
            serial::write("Chunk "); serial::write_hex(i as u32);
            serial::write("\n    offset: "); serial::write_hex(chunk.offset as u32);
            serial::write("\n    size: "); serial::write_hex(chunk.size as u32);
            serial::write("\n    free: ");
            serial::write(if chunk.free {
                "true"
            } else {
                "false"
            });
            serial::write("\n");
        }
    }
}

struct Allocator<const N: usize> {
    heap: UnsafeCell<Heap<N>>,
}

impl<const N: usize> Allocator<N> {
    pub const fn new() -> Self {
        Self {
            heap: UnsafeCell::new(Heap::new())
        }
    }

    const fn get_heap(&self) -> &mut Heap<N> {
        unsafe { self.heap.get().as_mut().unwrap() }
    }
}

unsafe impl<const N: usize> GlobalAlloc for Allocator<N> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let heap = self.get_heap();

        if let Some(idx) = heap.chunks.iter()
            .position(|chunk| heap.useable_chunk(chunk, layout))
        {
            heap.layout_chunk(idx, layout)
        } else {
            let align_offset = heap.align_offset(heap.cur_size, layout);
            let offset = heap.cur_size;
            heap.add_size(align_offset + layout.size());
            heap.push_chunk(Chunk {
                offset,
                size: align_offset + layout.size(),
                free: true,
            });
            heap.layout_chunk(heap.chunks.len() - 1, layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
        let heap = self.get_heap();

        let idx = heap.chunks.iter()
            .position(|chunk| heap.chunk_to_ptr(chunk) == ptr)
            .unwrap();
        let chunk = &mut heap.chunks[idx];
        chunk.free = true;

        heap.defrag();
    }
}

unsafe impl<const N: usize> Sync for Allocator<N> {}
