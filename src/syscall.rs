//! Generic syscalls for Windows, everything in here should be architecture
//! agnostic.

#[cfg(target_arch = "mips")] pub mod mips;
#[cfg(target_arch = "mips")] pub use mips::*;

use core::mem::MaybeUninit;
use core::ptr::{addr_of, addr_of_mut};
use core::cell::UnsafeCell;
use alloc::sync::Arc;
use alloc::boxed::Box;

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

/// Handle to a thread and a pointer to the return value
pub struct JoinHandle<T>(Handle, Arc<UnsafeCell<MaybeUninit<T>>>);

impl<T> JoinHandle<T> {
    pub fn join(self) -> Result<T> {
        // Wait for thread to exit
        wait(self.0)?;
       
        close(self.0);

        let usc = Arc::try_unwrap(self.1).map_err(|_| ()).unwrap();
        let inner = usc.into_inner();
        Ok(unsafe { inner.assume_init() })
    }
}

/// Spawn a thread
pub fn spawn<F, T>(f: F) -> Result<JoinHandle<T>>
        where F: FnOnce() -> T,
              F: Send + 'static,
              T: Send + 'static {
    // Holder for returned client handle
    let mut handle = 0usize;

    // Placeholder for returned client ID
    let mut client_id = [0usize; 2];

    // Create a new context
    let mut context: Context = unsafe { core::mem::zeroed() };

    // Allocate and leak a stack for the thread
    let stack = vec![0u8; 4096].leak();

    // Initial TEB, maybe some stack stuff in here!?
    let initial_teb = [0u32; 5];

    // External thread entry point
    extern fn entry<F, T>(func: *mut F,
                          ret:  *mut UnsafeCell<MaybeUninit<T>>) -> !
            where F: FnOnce() -> T,
                  F: Send + 'static,
                  T: Send + 'static {
        {
            // Re-box the FFI'd type
            let func: Box<F> = unsafe {
                Box::from_raw(func)
            };

            // Re-box the return type
            let ret: Arc<UnsafeCell<MaybeUninit<T>>> = unsafe {
                Arc::from_raw(ret)
            };

            // Call the closure and save the return
            unsafe { (&mut *ret.get()).write(func()); }
        }

        // Exit the thread
        exit_thread(0);
    }

    let rbox = unsafe {
        /// Control context
        const CONTEXT_CONTROL: u32 = 1;

        /// Floating point context
        const CONTEXT_FLOATING_POINT: u32 = 2;

        /// Integer context
        const CONTEXT_INTEGER: u32 = 4;

        // Set the flags for the registers we want to control
        context.context.bits64.flags =
            CONTEXT_CONTROL | CONTEXT_FLOATING_POINT | CONTEXT_INTEGER;

        // Thread entry point
        context.context.bits64.fir = entry::<F, T> as u32;

        // Set `$a0` argument
        let cbox: *mut F = Box::into_raw(Box::new(f));
        context.context.bits64.int[4] = cbox as u64;
        
        // Create return storage
        let rbox: Arc<UnsafeCell<MaybeUninit<T>>> =
            Arc::new(UnsafeCell::new(MaybeUninit::uninit()));
        context.context.bits64.int[5] = Arc::into_raw(rbox.clone()) as u64;

        // Set the 64-bit `$sp` to the end of the stack
        context.context.bits64.int[29] =
            stack.as_mut_ptr() as u64 + stack.len() as u64;
        
        rbox
    };

    // Create the thread
    let status = NtStatus(unsafe {
        syscall8(
            // OUT PHANDLE ThreadHandle
            addr_of_mut!(handle) as usize,

            // IN ACCESS_MASK DesiredAccess
            0x1f03ff,

            // IN POBJECT_ATTRIBUTES ObjectAttributes OPTIONAL
            0,

            // IN HANDLE ProcessHandle
            !0,

            // OUT PCLIENT_ID ClientId
            addr_of_mut!(client_id) as usize,

            // IN PCONTEXT ThreadContext,
            addr_of!(context) as usize,

            // IN PINITIAL_TEB InitialTeb
            addr_of!(initial_teb) as usize,

            // IN BOOLEAN CreateSuspended
            0,

            // Syscall number
            Syscall::CreateThread
        )
    } as u32);

    // Convert error to Rust error
    if status.success() {
        Ok(JoinHandle(Handle(handle), rbox))
    } else {
        Err(status)
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
fn close(handle: Handle) {
    unsafe {
        syscall1(handle.0, Syscall::Close);
    }
}

/// Exit the current thread with `code` as the exit status
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

