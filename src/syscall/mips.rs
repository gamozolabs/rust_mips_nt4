//! MIPS NT syscall conventions
//! Ref: https://devblogs.microsoft.com/oldnewthing/20180417-00/?p=98525
//!
//! Very similar to MIPS o32 convention, with interleaved floats and skipped
//! integer registers when floats are used.
//!
//! These wrappers use Rust naked functions and the fact that the o32 Rust-emit
//! C ABI matches the Windows kernel ABI. This allows us to simply move the
//! last parameter (syscall number in our Rust bindings) into the correct
//! syscall ID register `$v0` and pass through all existing parameters. This
//! decreases the amount of overhead and means we don't have to worry about
//! things like register homing and stack alignment as those are handled for
//! us.
//!
//! It also may be a bit confusing why we don't `ret` from the `syscall`, this
//! is because `syscall` on MIPS actually returns to the user-provided `$lr`,
//! meaning the `$lr` is set from the call to the naked function, thus the
//! syscall directly returns back to the caller of the `syscall*()` wrapper
//! function rather than to the instruction following the `syscall`

use core::mem::MaybeUninit;
use core::ptr::{addr_of, addr_of_mut};
use core::cell::UnsafeCell;
use alloc::sync::Arc;
use alloc::boxed::Box;
use super::{NtStatus, Result, Handle, JoinHandle, MEM_RELEASE};

/// Syscall numbers
#[allow(dead_code)]
#[repr(usize)]
pub enum Syscall {
    /// NtAllocateVirtualMemory()
    AllocateVirtualMemory = 0xa,

    /// NtClose()
    Close = 0xf,

    /// NtCreateThread()
    CreateThread = 0x24,

    /// NtFreeVirtualMemory()
    FreeVirtualMemory = 0x3a,

    /// NtOpenFile()
    OpenFile = 0x4f,

    /// NtTerminateProcess()
    TerminateProcess = 0xba,

    /// NtTerminateThread()
    TerminateThread = 0xbb,

    /// NtWaitForSingleObject()
    WaitForSingleObject = 0xc4,
    
    /// NtWriteFile()
    WriteFile = 0xc7,
}

/// Alignment structure for [`Context`]
#[repr(C)]
pub union ContextAlign {
    /// Argument?
    argument: u128,
}

/// Union of different bitness contexts
#[repr(C)]
pub union ContextBits {
    /// 32-bit context
    pub bits32: Context32,

    /// 64-bit context
    pub bits64: Context64,
}

/// Bitness-agnostic `_CONTEXT`
#[repr(C)]
pub struct Context {
    /// Alignment structure
    _align: ContextAlign,

    /// Contexts
    pub context: ContextBits,
}

/// 32-bit `_CONTEXT`
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Context32 {
    /// Floating point registers
    pub fp: [u32; 32],

    /// Integer registers
    pub int: [u32; 34],

    /// Status register?
    pub fsr: u32,
    
    /// Fault instruction continuation address
    pub fir: u32,

    /// Processor status
    pub psr: u32,

    /// Context update flags
    pub flags: u32,
}

/// 64-bit `_CONTEXT`
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Context64 {
    /// Floating point registers
    pub fp: [u64; 32],

    /// Filler
    pub _fill1: u32,

    /// Filler
    pub _fill2: u32,

    /// Status register?
    pub fsr: u32,
    
    /// Fault instruction continuation address
    pub fir: u32,

    /// Processor status
    pub psr: u32,

    /// Context update flags
    pub flags: u32,

    /// Integer registers
    pub int: [u64; 34],
}

/// 0-argument syscall
#[allow(unused)]
#[naked]
pub unsafe extern fn syscall0(id: Syscall) -> usize {
    asm!(r#"
        move $v0, $a0
        syscall
    "#, options(noreturn));
}

/// 1-argument syscall
#[allow(unused)]
#[naked]
pub unsafe extern fn syscall1(_: usize, id: Syscall) -> usize {
    asm!(r#"
        move $v0, $a1
        syscall
    "#, options(noreturn));
}

/// 2-argument syscall
#[allow(unused)]
#[naked]
pub unsafe extern fn syscall2(_: usize, _: usize, id: Syscall) -> usize {
    asm!(r#"
        move $v0, $a2
        syscall
    "#, options(noreturn));
}

/// 3-argument syscall
#[allow(unused)]
#[naked]
pub unsafe extern fn syscall3(_: usize, _: usize, _: usize, id: Syscall)
        -> usize {
    asm!(r#"
        move $v0, $a3
        syscall
    "#, options(noreturn));
}

/// 4-argument syscall
#[allow(unused)]
#[naked]
pub unsafe extern fn syscall4(_: usize, _: usize, _: usize, _: usize,
        id: Syscall) -> usize {
    asm!(r#"
        lw $v0, 0x10($sp)
        syscall
    "#, options(noreturn));
}

/// 5-argument syscall
#[allow(unused)]
#[naked]
pub unsafe extern fn syscall5(_: usize, _: usize, _: usize, _: usize,
        _: usize, id: Syscall) -> usize {
    asm!(r#"
        lw $v0, 0x14($sp)
        syscall
    "#, options(noreturn));
}

/// 6-argument syscall
#[allow(unused)]
#[naked]
pub unsafe extern fn syscall6(_: usize, _: usize, _: usize, _: usize,
        _: usize, _: usize, id: Syscall) -> usize {
    asm!(r#"
        lw $v0, 0x18($sp)
        syscall
    "#, options(noreturn));
}

/// 7-argument syscall
#[allow(unused)]
#[naked]
pub unsafe extern fn syscall7(_: usize, _: usize, _: usize, _: usize,
        _: usize, _: usize, _: usize, id: Syscall) -> usize {
    asm!(r#"
        lw $v0, 0x1c($sp)
        syscall
    "#, options(noreturn));
}

/// 8-argument syscall
#[allow(unused)]
#[naked]
pub unsafe extern fn syscall8(_: usize, _: usize, _: usize, _: usize,
        _: usize, _: usize, _: usize, _: usize, id: Syscall) -> usize {
    asm!(r#"
        lw $v0, 0x20($sp)
        syscall
    "#, options(noreturn));
}

/// 9-argument syscall
#[allow(unused)]
#[naked]
pub unsafe extern fn syscall9(_: usize, _: usize, _: usize, _: usize,
        _: usize, _: usize, _: usize, _: usize, _: usize, id: Syscall)
        -> usize {
    asm!(r#"
        lw $v0, 0x24($sp)
        syscall
    "#, options(noreturn));
}

/// Spawn a thread
///
/// MIPS specific due to some inline assembly as well as MIPS-specific context
/// structure creation.
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

    /// External thread entry point
    extern fn entry<F, T>(func:      *mut F,
                          ret:       *mut UnsafeCell<MaybeUninit<T>>,
                          mut stack:  usize) -> !
            where F: FnOnce() -> T,
                  F: Send + 'static,
                  T: Send + 'static {
        // Create a scope so that we drop `Box` and `Arc`
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

        // Region size
        let mut rsize = 0usize;

        unsafe {
            asm!(r#"
                // Set the link register
                jal 2f

                // Exit thread
                jal 3f
                break

            2:
                // NtFreeVirtualMemory()
                li $v0, {free}
                syscall

            3:
                // NtTerminateThread()
                li $v0, {terminate}
                li $a0, -2 // GetCurrentThread()
                li $a1, 0  // exit code
                syscall

            "#, terminate = const Syscall::TerminateThread   as usize,
                free      = const Syscall::FreeVirtualMemory as usize,
                in("$4") !0usize,
                in("$5") addr_of_mut!(stack),
                in("$6") addr_of_mut!(rsize),
                in("$7") MEM_RELEASE, options(noreturn));
        }
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
        context.context.bits64.fir = entry::<F, T> as usize as u32;

        // Set `$a0` argument
        let cbox: *mut F = Box::into_raw(Box::new(f));
        context.context.bits64.int[4] = cbox as u64;
        
        // Create return storage in `$a1`
        let rbox: Arc<UnsafeCell<MaybeUninit<T>>> =
            Arc::new(UnsafeCell::new(MaybeUninit::uninit()));
        context.context.bits64.int[5] = Arc::into_raw(rbox.clone()) as u64;

        // Pass in stack in `$a2`
        context.context.bits64.int[6] = stack.as_mut_ptr() as u64;

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

