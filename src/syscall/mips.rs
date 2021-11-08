/// Syscall numbers
#[repr(usize)]
pub enum Syscall {
    /// NtOpenFile()
    NtOpenFile = 0x4f,
    
    /// NtWriteFile()
    NtWriteFile = 0xc7,

    /// NtTerminateProcess()
    NtTerminateProcess = 0xba,
}

/// 2-argument syscall
#[inline(never)]
pub unsafe extern fn syscall2(arg1: usize, arg2: usize, id: Syscall) -> usize {
    let ret: usize;
    asm!(r#"
        syscall
    "#, inout("$2") (id as usize) => ret);
    ret
}

/// 6-argument syscall
#[inline(never)]
pub unsafe extern fn syscall6(
        arg1: usize, arg2: usize,
        arg3: usize, arg4: usize, arg5: usize, arg6: usize,
        id: Syscall) -> usize {
    let ret: usize;
    asm!(r#"
        syscall
    "#, inout("$2") (id as usize) => ret);
    ret
}

/// 9-argument syscall
#[inline(never)]
pub unsafe extern fn syscall9(
        arg1: usize, arg2: usize,
        arg3: usize, arg4: usize, arg5: usize, arg6: usize,
        arg7: usize, arg8: usize, arg9: usize,
        id: Syscall) -> usize {
    let ret: usize;
    asm!(r#"
        syscall
    "#, inout("$2") (id as usize) => ret);
    ret
}

