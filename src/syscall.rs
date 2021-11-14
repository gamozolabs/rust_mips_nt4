//! Generic syscalls for Windows, everything in here should be architecture
//! agnostic.

#[cfg(target_arch = "mips")] pub mod mips;
#[cfg(target_arch = "mips")] pub use mips::*;

use core::ptr::addr_of_mut;

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
#[derive(Clone, Copy, Debug, Default)]
#[repr(transparent)]
pub struct Handle(pub usize);

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

/// De-allocate virtual memory in the current process
pub unsafe fn munmap(mut addr: usize) -> Result<()> {
    /// Release memory range
    const MEM_RELEASE: u32 = 0x8000;

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
pub fn write(fd: Handle, bytes: impl AsRef<[u8]>) -> Result<usize> {
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

/// Exit the current process with `code` as the exit status
pub fn exit(code: usize) -> ! {
    unsafe {
        syscall2(!0, code, Syscall::TerminateProcess);
        core::hint::unreachable_unchecked();
    }
}

