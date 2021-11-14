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

