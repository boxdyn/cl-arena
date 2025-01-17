//! A [DroplessArena] can hold *any* combination of types as long as they don't implement
//! [Drop].
use crate::{chunk::ArenaChunk, constants::*};
use alloc::vec::Vec;
use core::{
    alloc::Layout,
    cell::{Cell, RefCell},
    marker::PhantomData,
    mem, ptr, slice,
};

pub struct DroplessArena<'arena> {
    _lives: PhantomData<&'arena u8>,
    chunks: RefCell<Vec<ArenaChunk<u8>>>,
    head: Cell<*mut u8>,
    tail: Cell<*mut u8>,
}

impl Default for DroplessArena<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'arena> DroplessArena<'arena> {
    pub const fn new() -> Self {
        Self {
            _lives: PhantomData,
            chunks: RefCell::new(Vec::new()),
            head: Cell::new(ptr::null_mut()),
            tail: Cell::new(ptr::null_mut()),
        }
    }

    /// Allocates a `T` in the [DroplessArena], and returns a mutable reference to it.
    ///
    /// # Panics
    /// - Panics if T implements [Drop]
    /// - Panics if T is zero-sized
    #[allow(clippy::mut_from_ref)]
    pub fn alloc<T>(&'arena self, value: T) -> &'arena mut T {
        assert!(!mem::needs_drop::<T>());
        assert!(mem::size_of::<T>() != 0);

        let out = self.alloc_raw(Layout::new::<T>()) as *mut T;

        unsafe {
            ptr::write(out, value);
            &mut *out
        }
    }

    /// Allocates a slice of `T`s`, copied from the given slice, returning a mutable reference
    /// to it.
    ///
    /// # Panics
    /// - Panics if T implements [Drop]
    /// - Panics if T is zero-sized
    /// - Panics if the slice is empty
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_slice<T: Copy>(&'arena self, slice: &[T]) -> &'arena mut [T] {
        assert!(!mem::needs_drop::<T>());
        assert!(mem::size_of::<T>() != 0);
        assert!(!slice.is_empty());

        let mem = self.alloc_raw(Layout::for_value::<[T]>(slice)) as *mut T;

        unsafe {
            mem.copy_from_nonoverlapping(slice.as_ptr(), slice.len());
            slice::from_raw_parts_mut(mem, slice.len())
        }
    }

    /// Allocates a copy of the given [`&str`](str), returning a reference to the allocation.
    ///
    /// # Panics
    /// Panics if the string is empty.
    pub fn alloc_str(&'arena self, string: &str) -> &'arena str {
        let slice = self.alloc_slice(string.as_bytes());

        // Safety: This is a clone of the input string, which was valid
        unsafe { core::str::from_utf8_unchecked(slice) }
    }

    /// Allocates some [bytes](u8) based on the given [Layout].
    ///
    /// # Panics
    /// Panics if the provided [Layout] has size 0
    pub fn alloc_raw(&'arena self, layout: Layout) -> *mut u8 {
        /// Rounds the given size (or pointer value) *up* to the given alignment
        fn align_up(size: usize, align: usize) -> usize {
            (size + align - 1) & !(align - 1)
        }
        /// Rounds the given size (or pointer value) *down* to the given alignment
        fn align_down(size: usize, align: usize) -> usize {
            size & !(align - 1)
        }

        assert!(layout.size() != 0);
        loop {
            let Self { head, tail, .. } = self;
            let start = head.get().addr();
            let end = tail.get().addr();

            let align = 8.max(layout.align());

            let bytes = align_up(layout.size(), align);

            if let Some(end) = end.checked_sub(bytes) {
                let end = align_down(end, layout.align());

                if start <= end {
                    tail.set(tail.get().with_addr(end));
                    return tail.get();
                }
            }

            self.grow(layout.size());
        }
    }

    /// Grows the allocator, doubling the chunk size until it reaches [MAX_CHUNK].
    #[cold]
    #[inline(never)]
    fn grow(&self, len: usize) {
        let mut chunks = self.chunks.borrow_mut();

        let capacity = if let Some(last) = chunks.last_mut() {
            last.mem.len().min(MAX_CHUNK / 2) * 2
        } else {
            MIN_CHUNK
        }
        .max(len);

        let mut chunk = ArenaChunk::<u8>::new(capacity);

        self.head.set(chunk.start());
        self.tail.set(chunk.end());
        chunks.push(chunk);
    }

    /// Checks whether the given slice is allocated in this arena
    pub fn contains_slice<T>(&self, slice: &[T]) -> bool {
        let ptr = slice.as_ptr().cast::<u8>().cast_mut();
        for chunk in self.chunks.borrow_mut().iter_mut() {
            if chunk.start() <= ptr && ptr <= chunk.end() {
                return true;
            }
        }
        false
    }
}

unsafe impl Send for DroplessArena<'_> {}

#[cfg(test)]
mod tests;
