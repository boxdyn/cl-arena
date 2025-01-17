//! A [TypedArena] can hold many instances of a single type, and will properly [Drop] them.
#![allow(clippy::mut_from_ref)]

use crate::{chunk::ArenaChunk, constants::*};
use alloc::vec::Vec;
use core::{
    cell::{Cell, RefCell},
    marker::PhantomData,
    mem, ptr, slice,
};

/// A [TypedArena] can hold many instances of a single type, and will properly [Drop] them when
/// it falls out of scope.
pub struct TypedArena<'arena, T> {
    _lives: PhantomData<&'arena T>,
    _drops: PhantomData<T>,
    chunks: RefCell<Vec<ArenaChunk<T>>>,
    head: Cell<*mut T>,
    tail: Cell<*mut T>,
}

impl<T> Default for TypedArena<'_, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'arena, T> TypedArena<'arena, T> {
    pub const fn new() -> Self {
        Self {
            _lives: PhantomData,
            _drops: PhantomData,
            chunks: RefCell::new(Vec::new()),
            head: Cell::new(ptr::null_mut()),
            tail: Cell::new(ptr::null_mut()),
        }
    }

    pub fn alloc(&'arena self, value: T) -> &'arena mut T {
        if self.head == self.tail {
            self.grow(1);
        }

        let out = if mem::size_of::<T>() == 0 {
            self.head
                .set(ptr::without_provenance_mut(self.head.get().addr() + 1));
            ptr::NonNull::<T>::dangling().as_ptr()
        } else {
            let out = self.head.get();
            self.head.set(unsafe { out.add(1) });
            out
        };

        unsafe {
            ptr::write(out, value);
            &mut *out
        }
    }

    fn can_allocate(&self, len: usize) -> bool {
        len <= unsafe { self.tail.get().offset_from(self.head.get()) as usize }
    }

    /// # Panics
    /// Panics if size_of::<T> == 0 || len == 0
    #[inline]
    fn alloc_raw_slice(&self, len: usize) -> *mut T {
        assert!(mem::size_of::<T>() != 0);
        assert!(len != 0);

        if !self.can_allocate(len) {
            self.grow(len)
        }

        let out = self.head.get();

        unsafe { self.head.set(out.add(len)) };
        out
    }

    pub fn alloc_from_iter<I>(&'arena self, iter: I) -> &'arena mut [T]
    where
        I: IntoIterator<Item = T>,
    {
        // Collect them all into a buffer so they're allocated contiguously
        let mut buf = iter.into_iter().collect::<Vec<_>>();
        if buf.is_empty() {
            return &mut [];
        }

        let len = buf.len();
        // If T is a ZST, calling alloc_raw_slice will panic
        let slice = if mem::size_of::<T>() == 0 {
            self.head
                .set(ptr::without_provenance_mut(self.head.get().addr() + len));
            ptr::NonNull::dangling().as_ptr()
        } else {
            self.alloc_raw_slice(len)
        };

        unsafe {
            buf.as_ptr().copy_to_nonoverlapping(slice, len);
            buf.set_len(0);
            slice::from_raw_parts_mut(slice, len)
        }
    }

    #[cold]
    #[inline(never)]
    fn grow(&self, len: usize) {
        let size = mem::size_of::<T>().max(1);

        let mut chunks = self.chunks.borrow_mut();

        let capacity = if let Some(last) = chunks.last_mut() {
            last.filled = self.get_filled_of_chunk(last);
            last.mem.len().min(MAX_CHUNK / size) * 2
        } else {
            MIN_CHUNK / size
        }
        .max(len);

        let mut chunk = ArenaChunk::<T>::new(capacity);

        self.head.set(chunk.start());
        self.tail.set(chunk.end());
        chunks.push(chunk);
    }

    fn get_filled_of_chunk(&self, chunk: &mut ArenaChunk<T>) -> usize {
        let Self { head: tail, .. } = self;
        let head = chunk.start();
        if mem::size_of::<T>() == 0 {
            tail.get().addr() - head.addr()
        } else {
            unsafe { tail.get().offset_from(head) as usize }
        }
    }
}

unsafe impl<T: Send> Send for TypedArena<'_, T> {}

unsafe impl<#[may_dangle] T> Drop for TypedArena<'_, T> {
    fn drop(&mut self) {
        let mut chunks = self.chunks.borrow_mut();

        if let Some(last) = chunks.last_mut() {
            last.filled = self.get_filled_of_chunk(last);
            self.tail.set(self.head.get());
        }

        for chunk in chunks.iter_mut() {
            unsafe { chunk.drop_elements() }
        }
    }
}

#[cfg(test)]
mod tests;
