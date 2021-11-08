#[cfg(target_arch = "mips")] pub mod mips;
#[cfg(target_arch = "mips")] pub use mips::*;

/// Exit the current process
pub fn exit(code: usize) -> ! {
    unsafe {
        syscall2(!0, code, Syscall::NtTerminateProcess);
        core::hint::unreachable_unchecked();
    }
}

