//! Memory manager

use alloc::alloc::{Layout, GlobalAlloc};

/// Implementation of the global allocator
struct GlobalAllocator;

/// Global allocator object
#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator;

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        crate::syscall::mmap(0, layout.size()).unwrap_or(core::ptr::null_mut())
    }
    
    unsafe fn dealloc(&self, addr: *mut u8, _layout: Layout) {
        crate::syscall::munmap(addr as usize)
            .expect("Failed to deallocate memory");
    }
}

