//! Generic syscalls for Windows

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

/// Write to a file
pub fn write(fd: Handle, bytes: impl AsRef<[u8]>) -> Result<usize> {
    let mut offset = 0u64;
    let mut iosb = IoStatusBlock::default();

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

