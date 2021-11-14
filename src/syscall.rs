//! Generic syscalls for Windows, everything in here should be architecture
//! agnostic.

#[cfg(target_arch = "mips")] pub mod mips;
#[cfg(target_arch = "mips")] pub use mips::*;

use core::mem::MaybeUninit;
use core::ptr::addr_of_mut;
use core::cell::UnsafeCell;
use alloc::sync::Arc;

/// Attempted a syscall which may have failed with `NtStatus`
pub type Result<T> = core::result::Result<T, NtStatus>;

/// `NTSTATUS`
#[derive(Clone, Copy, Debug, Default)]
#[repr(transparent)]
pub struct NtStatus(pub u32);

impl NtStatus {
    /// Returns `true` if the status was successful
    pub fn success(self) -> bool {
        (self.0 as i32) >= 0
    }
}

/// `HANDLE`
#[derive(Debug)]
#[repr(transparent)]
pub struct Handle(usize);

impl Handle {
    /// Create a handle out of thin air
    pub const unsafe fn from_raw(raw: usize) -> Self {
        Self(raw)
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        close(self).expect("Failed to close handle");
    }
}

/// `IO_STATUS_BLOCK`
#[derive(Default)]
#[repr(C)]
pub struct IoStatusBlock {
    /// Status code from the command
    pub status: NtStatus,

    /// Request-dependent value about the request
    pub information: usize,
}

/// Allocate virtual memory in the current process
pub fn mmap(mut addr: usize, mut size: usize) -> Result<*mut u8> {
    /// Commit memory
    const MEM_COMMIT: u32 = 0x1000;

    /// Reserve memory range
    const MEM_RESERVE: u32 = 0x2000;

    /// Readable and writable memory
    const PAGE_READWRITE: u32 = 0x4;

    // Perform syscall
    let status = NtStatus(unsafe {
        syscall6(
            // [in] HANDLE ProcessHandle,
            !0,

            // [in, out] PVOID *BaseAddress,
            addr_of_mut!(addr) as usize,

            // [in] ULONG_PTR ZeroBits,
            0,

            // [in, out] PSIZE_T RegionSize,
            addr_of_mut!(size) as usize,

            // [in] ULONG AllocationType,
            (MEM_COMMIT | MEM_RESERVE) as usize,

            // [in] ULONG Protect
            PAGE_READWRITE as usize,

            // Syscall ID
            Syscall::AllocateVirtualMemory,
        )
    } as u32);

    // If success, return allocation otherwise return status
    if status.success() {
        Ok(addr as *mut u8)
    } else {
        Err(status)
    }
}
    
/// Release memory range
const MEM_RELEASE: u32 = 0x8000;

/// De-allocate virtual memory in the current process
pub unsafe fn munmap(mut addr: usize) -> Result<()> {
    // Region size
    let mut size = 0usize;

    // Perform syscall
    let status = NtStatus(syscall4(
        // [in] HANDLE ProcessHandle,
        !0,

        // [in, out] PVOID *BaseAddress,
        addr_of_mut!(addr) as usize,

        // [in, out] PSIZE_T RegionSize,
        addr_of_mut!(size) as usize,

        // [in] ULONG AllocationType,
        MEM_RELEASE as usize,

        // Syscall ID
        Syscall::FreeVirtualMemory,
    ) as u32);

    // Return error on error
    if status.success() {
        Ok(())
    } else {
        Err(status)
    }
}

/// Write to a file
pub fn write(fd: &Handle, bytes: impl AsRef<[u8]>) -> Result<usize> {
    let mut offset = 0u64;
    let mut iosb = IoStatusBlock::default();

    // Perform syscall
    let status = NtStatus(unsafe {
        syscall9(
            // [in] HANDLE FileHandle
            fd.0,

            // [in, optional] HANDLE Event
            0,

            // [in, optional] PIO_APC_ROUTINE ApcRoutine,
            0,

            // [in, optional] PVOID ApcContext,
            0,

            // [out] PIO_STATUS_BLOCK IoStatusBlock,
            addr_of_mut!(iosb) as usize,

            // [in] PVOID Buffer,
            bytes.as_ref().as_ptr() as usize,

            // [in] ULONG Length,
            bytes.as_ref().len(),

            // [in, optional] PLARGE_INTEGER ByteOffset,
            addr_of_mut!(offset) as usize,

            // [in, optional] PULONG Key
            0,

            // Syscall number
            Syscall::WriteFile)
    } as u32);

    // If success, return number of bytes written, otherwise return error
    if status.success() {
        Ok(iosb.information)
    } else {
        Err(status)
    }
}

/// Handle to a thread and a pointer to the return value
pub struct JoinHandle<T>(Handle, Arc<UnsafeCell<MaybeUninit<T>>>);

impl<T> JoinHandle<T> {
    /// Block until the thread exits and return the return value from the
    /// thread.
    pub fn join(self) -> Result<T> {
        // Wait for thread to exit
        wait(self.0)?;

        // Try to unwrap the `Arc`, this is only possible if the thread has
        // exited (thus, there is only one reference to the `Arc`). This
        // atomically double-checks that not only has the thread exited because
        // the `wait()` above succeeded, but also the `Arc` was dropped inside
        // the thread
        let usc = Arc::try_unwrap(self.1).map_err(|_| ()).unwrap();

        // Now that we have exclusive access to the return value, we can get
        // the inner part of the `UnsafeCell` and assume it is initialized.
        // It is impossible for the `Arc` in the thread to have been dropped
        // without initializing the value, thus this is safe.
        let inner = usc.into_inner();

        // Assume init!
        Ok(unsafe { inner.assume_init() })
    }
}

/// Block on a handle forever
pub fn wait(handle: Handle) -> Result<()> {
    let status = NtStatus(unsafe {
        syscall3(
            // [in] HANDLE Handle,
            handle.0 as usize,

            // [in] BOOLEAN Alertable,
            0,

            // [in, optional] PLARGE_INTEGER Timeout
            0,
            Syscall::WaitForSingleObject
        )
    } as u32);

    // Convert error to Rust error
    if status.success() {
        Ok(())
    } else {
        Err(status)
    }
}

/// Close a handle
fn close(handle: &Handle) -> Result<()> {
    // Close the handle
    let status = NtStatus(unsafe {
        syscall1(handle.0, Syscall::Close)
    } as u32);
    
    // Convert error to Rust error
    if status.success() {
        Ok(())
    } else {
        Err(status)
    }
}

/// Exit the current thread with `code` as the exit status
#[allow(dead_code)]
pub fn exit_thread(code: usize) -> ! {
    unsafe {
        syscall2((-2isize) as usize, code, Syscall::TerminateThread);
        core::hint::unreachable_unchecked();
    }
}

/// Exit the current process with `code` as the exit status
pub fn exit(code: usize) -> ! {
    unsafe {
        syscall2(!0, code, Syscall::TerminateProcess);
        core::hint::unreachable_unchecked();
    }
}

