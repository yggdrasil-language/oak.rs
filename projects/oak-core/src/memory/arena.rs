#![doc = include_str!("readme.md")]
use crate::tree::TokenProvenance;
use std::{
    alloc::{Layout, alloc, dealloc},
    cell::{RefCell, UnsafeCell},
    ptr::{NonNull, copy_nonoverlapping},
};

/// Default chunk size: 64KB.
/// Large enough to amortize the cost of system-level allocations, yet small enough to be
/// L2-cache friendly and avoid excessive internal fragmentation for small templates.
const CHUNK_SIZE: usize = 64 * 1024;

/// Alignment for all allocations: 8 bytes.
/// This covers all standard Rust primitives (u64, f64, pointers) on 64-bit architectures.
const ALIGN: usize = 8;

// A thread-local pool of memory chunks to avoid hitting the global allocator.
//
// **Memory Safety & Leak Prevention:**
// - **Bounded Capacity:** The pool is limited to 64 chunks (4MB) per thread to prevent
//   unbounded memory growth in long-running processes.
// - **Large Allocations:** Chunks larger than `CHUNK_SIZE` (64KB) are not pooled and
//   are returned directly to the global allocator upon drop.
// - **Automatic Cleanup:** All chunks are either recycled into this pool or freed when
//   the `SyntaxArena` is dropped.
thread_local! {
    static CHUNK_POOL: RefCell<Vec<NonNull<u8>>> = RefCell::new(Vec::with_capacity(16))
}

/// A high-performance bump allocator optimized for AST nodes.
///
/// The arena works by "bumping" a pointer within a pre-allocated chunk of memory.
/// When a chunk is exhausted, a new one is requested from the thread-local pool
/// or the global allocator.
pub struct SyntaxArena {
    /// Pointer to the next free byte in the current chunk.
    /// Always kept 8-byte aligned.
    ptr: UnsafeCell<NonNull<u8>>,
    /// Pointer to the end of the current chunk (exclusive).
    end: UnsafeCell<NonNull<u8>>,
    /// List of full chunks allocated by this arena (excluding the current one).
    /// Stored as (start_pointer, total_size).
    full_chunks: UnsafeCell<Vec<(NonNull<u8>, usize)>>,
    /// The start pointer of the current chunk (used for recycling/freeing).
    current_chunk_start: UnsafeCell<NonNull<u8>>,
    /// Store for token provenance metadata.
    metadata: UnsafeCell<Vec<TokenProvenance>>,
}

impl SyntaxArena {
    /// Creates a new empty arena.
    ///
    /// Initial pointers are set to dangling. The first allocation will trigger
    /// a chunk allocation.
    pub fn new(capacity: usize) -> Self {
        // Use a pointer aligned to ALIGN even for the dangling state to satisfy debug assertions.
        let dangling = unsafe { NonNull::new_unchecked(ALIGN as *mut u8) };
        Self { ptr: UnsafeCell::new(dangling), end: UnsafeCell::new(dangling), full_chunks: UnsafeCell::new(Vec::with_capacity(capacity)), current_chunk_start: UnsafeCell::new(NonNull::dangling()), metadata: UnsafeCell::new(Vec::new()) }
    }

    /// Stores a token provenance in the arena and returns its index.
    pub fn add_metadata(&self, provenance: TokenProvenance) -> std::num::NonZeroU32 {
        let metadata = unsafe { &mut *self.metadata.get() };
        metadata.push(provenance);
        std::num::NonZeroU32::new(metadata.len() as u32).expect("Metadata index overflow")
    }

    /// Retrieves a token provenance by index.
    pub fn get_metadata(&self, index: std::num::NonZeroU32) -> Option<&TokenProvenance> {
        let metadata = unsafe { &*self.metadata.get() };
        metadata.get(index.get() as usize - 1)
    }

    /// Allocates a value of type `T` in the arena and moves `value` into it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `T` is a POD (Plain Old Data) type.
    /// The `Drop` implementation for `T` (if any) will **not** be called when
    /// the arena is dropped.
    ///
    /// # Panics
    ///
    /// Panics if the allocation fails (OOM).
    #[inline(always)]
    pub fn alloc<T>(&self, value: T) -> &mut T {
        let layout = Layout::new::<T>();
        // Ensure the type's alignment requirement is within our 8-byte guarantee.
        debug_assert!(layout.align() <= ALIGN);

        unsafe {
            let ptr = self.alloc_raw(layout.size());
            let ptr = ptr.as_ptr() as *mut T;
            // Write the value into the allocated space.
            ptr.write(value);
            &mut *ptr
        }
    }

    /// Allocates a slice in the arena and copies the contents of `slice` into it.
    ///
    /// This is useful for storing strings or other contiguous data in the arena.
    ///
    /// # Safety
    ///
    /// Same as `alloc`, `T` must be `Copy` (and thus POD).
    #[inline(always)]
    pub fn alloc_slice_copy<T: Copy>(&self, slice: &[T]) -> &mut [T] {
        if slice.is_empty() {
            return &mut [];
        }
        let layout = Layout::for_value(slice);
        debug_assert!(layout.align() <= ALIGN);

        unsafe {
            let ptr = self.alloc_raw(layout.size());
            let ptr = ptr.as_ptr() as *mut T;
            copy_nonoverlapping(slice.as_ptr(), ptr, slice.len());
            std::slice::from_raw_parts_mut(ptr, slice.len())
        }
    }

    /// Allocates a slice in the arena and fills it using an iterator.
    ///
    /// This is more efficient than collecting into a temporary `Vec` and then copying,
    /// as it writes directly into the arena memory.
    ///
    /// # Safety
    ///
    /// The iterator must yield exactly `count` items. If it yields fewer, the remaining
    /// memory will be uninitialized (UB if accessed). If it yields more, the extra
    /// items are ignored. `T` must be POD.
    #[inline(always)]
    pub fn alloc_slice_fill_iter<T, I>(&self, count: usize, iter: I) -> &mut [T]
    where
        I: IntoIterator<Item = T>,
    {
        if count == 0 {
            return &mut [];
        }
        let layout = Layout::array::<T>(count).unwrap();
        debug_assert!(layout.align() <= ALIGN);

        unsafe {
            let ptr = self.alloc_raw(layout.size());
            let base_ptr = ptr.as_ptr() as *mut T;

            let mut i = 0;
            for item in iter {
                if i >= count {
                    break;
                }
                base_ptr.add(i).write(item);
                i += 1
            }

            // In a production-ready system, we should handle the case where iter is short.
            // But for our internal use in deep_clone, we know the count is exact.
            debug_assert_eq!(i, count, "Iterator yielded fewer items than expected");

            std::slice::from_raw_parts_mut(base_ptr, count)
        }
    }

    /// Internal raw allocation logic.
    ///
    /// Attempts to allocate `size` bytes from the current chunk.
    /// If there is not enough space, it falls back to `alloc_slow`.
    ///
    /// # Safety
    ///
    /// `size` must be non-zero. The returned pointer is guaranteed to be 8-byte aligned.
    #[inline(always)]
    unsafe fn alloc_raw(&self, size: usize) -> NonNull<u8> {
        // Unsafe block to wrap unsafe ops
        unsafe {
            let ptr = *self.ptr.get();
            let end = *self.end.get();

            // Calculate aligned pointer. Since we always maintain ALIGN (8) byte alignment
            // for `self.ptr`, we only need to add the size and check against `end`.
            let current_addr = ptr.as_ptr() as usize;

            // Safety check: ensure the pointer is indeed aligned as we expect.
            debug_assert!(current_addr % ALIGN == 0);

            // We add `size` and then align up the result for the NEXT allocation.
            let next_addr = (current_addr + size + ALIGN - 1) & !(ALIGN - 1);

            if std::intrinsics::likely(next_addr <= end.as_ptr() as usize) {
                *self.ptr.get() = NonNull::new_unchecked(next_addr as *mut u8);
                return ptr;
            }

            self.alloc_slow(size)
        }
    }

    /// Slow path for allocation when the current chunk is exhausted.
    ///
    /// 1. Pushes the current chunk to `full_chunks`.
    /// 2. Allocates a new chunk (either standard 64KB or larger if `size` requires it).
    /// 3. Sets the new chunk as the current one.
    #[inline(never)]
    unsafe fn alloc_slow(&self, size: usize) -> NonNull<u8> {
        unsafe {
            // Retire current chunk if it exists.
            let current_start = *self.current_chunk_start.get();
            if current_start != NonNull::dangling() {
                // We record the full size of the chunk so it can be correctly recycled.
                // Note: for now we assume chunks are either CHUNK_SIZE or specially sized.
                let current_end = (*self.end.get()).as_ptr() as usize;
                let actual_size = current_end - current_start.as_ptr() as usize;
                (*self.full_chunks.get()).push((current_start, actual_size))
            }

            // Allocate new chunk.
            // If request is huge (> CHUNK_SIZE), we allocate a larger chunk specifically for it.
            // These "huge chunks" are NOT recycled into the pool to avoid wasting space.
            let alloc_size = usize::max(size + ALIGN, CHUNK_SIZE);

            let chunk_ptr = Self::alloc_chunk(alloc_size);

            *self.current_chunk_start.get() = chunk_ptr;

            let start_addr = chunk_ptr.as_ptr() as usize;
            // Resulting pointer is the start of the new chunk.
            let result_ptr = NonNull::new_unchecked(start_addr as *mut u8);

            // Calculate the next free pointer, aligned to ALIGN.
            let next_free = (start_addr + size + ALIGN - 1) & !(ALIGN - 1);

            *self.ptr.get() = NonNull::new_unchecked(next_free as *mut u8);
            *self.end.get() = NonNull::new_unchecked((start_addr + alloc_size) as *mut u8);

            result_ptr
        }
    }

    /// Allocates a new memory chunk from the thread-local pool or global allocator.
    unsafe fn alloc_chunk(size: usize) -> NonNull<u8> {
        // Try to get from pool if size matches the standard chunk size.
        if size == CHUNK_SIZE {
            let ptr = CHUNK_POOL.try_with(|pool| pool.borrow_mut().pop());

            if let Ok(Some(ptr)) = ptr {
                return ptr;
            }
        }

        let layout = Layout::from_size_align(size, ALIGN).unwrap();
        // unsafe block for alloc
        unsafe {
            let ptr = alloc(layout);
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout)
            }
            NonNull::new_unchecked(ptr)
        }
    }
}

impl Drop for SyntaxArena {
    /// Drops the arena, recycling all its chunks back to the thread-local pool or freeing them.
    fn drop(&mut self) {
        unsafe {
            // Recycle the current chunk.
            let current = *self.current_chunk_start.get();
            if current != NonNull::dangling() {
                let current_end = (*self.end.get()).as_ptr() as usize;
                let actual_size = current_end - current.as_ptr() as usize;
                Self::recycle_chunk(current, actual_size)
            }

            // Recycle all full chunks.
            for (ptr, size) in (*self.full_chunks.get()).iter() {
                Self::recycle_chunk(*ptr, *size)
            }
        }
    }
}

impl SyntaxArena {
    /// Returns a chunk to the thread-local pool or deallocates it if the pool is full.
    ///
    /// # Safety
    ///
    /// `ptr` must have been allocated with `ALIGN` and its size must be `size`.
    unsafe fn recycle_chunk(ptr: NonNull<u8>, size: usize) {
        if size == CHUNK_SIZE {
            // Only pool standard-sized chunks to maintain predictability.
            let _ = CHUNK_POOL
                .try_with(|pool| {
                    let mut pool = pool.borrow_mut();
                    if pool.len() < 64 {
                        // Hard limit to prevent memory bloating per thread.
                        pool.push(ptr)
                    }
                    else {
                        // If pool is full, deallocate the chunk
                        unsafe {
                            let layout = Layout::from_size_align(size, ALIGN).unwrap();
                            dealloc(ptr.as_ptr(), layout);
                        }
                    }
                })
                .unwrap_or_else(|_| {
                    // If try_with fails (e.g. during thread destruction), deallocate the chunk
                    unsafe {
                        let layout = Layout::from_size_align(size, ALIGN).unwrap();
                        dealloc(ptr.as_ptr(), layout);
                    }
                });
            return;
        }
        // If not pooled (either because it's a huge chunk or the pool is full/unreachable), deallocate immediately.
        let layout = Layout::from_size_align(size, ALIGN).unwrap();
        unsafe { dealloc(ptr.as_ptr(), layout) }
    }
}

unsafe impl Send for SyntaxArena {}
unsafe impl Sync for SyntaxArena {}

impl Default for SyntaxArena {
    fn default() -> Self {
        Self::new(16)
    }
}
