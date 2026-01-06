//! An [ArenaChunk] contains a block of raw memory for use in arena allocators.
use alloc::boxed::Box;
use core::{
    mem::{self, MaybeUninit},
    ptr::{self, NonNull},
};

pub struct ArenaChunk<T> {
    pub(crate) mem: NonNull<[MaybeUninit<T>]>,
    #[cfg(feature = "nightly")]
    pub(crate) filled: usize,
}

impl<T: Sized> ArenaChunk<T> {
    pub fn new(cap: usize) -> Self {
        let slice = Box::new_uninit_slice(cap);
        Self {
            mem: NonNull::from(Box::leak(slice)),
            #[cfg(feature = "nightly")]
            filled: 0,
        }
    }

    /// Drops all elements inside self, and resets the filled count to 0
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self.filled` elements of self are currently initialized
    #[cfg(feature = "nightly")]
    pub unsafe fn drop_elements(&mut self) {
        if mem::needs_drop::<T>() {
            // Safety: the caller has ensured that `filled` elements are initialized
            unsafe {
                let slice = self.mem.as_mut();
                for t in slice[..self.filled].iter_mut() {
                    t.assume_init_drop();
                }
            }
            self.filled = 0;
        }
    }

    /// Gets a pointer to the start of the arena
    pub fn start(&mut self) -> *mut T {
        self.mem.as_ptr() as _
    }

    /// Gets a pointer to the end of the arena
    pub fn end(&mut self) -> *mut T {
        if mem::size_of::<T>() == 0 {
            ptr::without_provenance_mut(usize::MAX) // pointers to ZSTs must be unique
        } else {
            unsafe { self.start().add(self.mem.len()) }
        }
    }
}

impl<T> Drop for ArenaChunk<T> {
    fn drop(&mut self) {
        let _ = unsafe { Box::from_raw(self.mem.as_ptr()) };
    }
}
