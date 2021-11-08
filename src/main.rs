#![feature(asm)]
#![no_std]
#![no_main]

use core::sync::atomic::{AtomicUsize, Ordering};

mod syscall;

#[panic_handler]
fn moose(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[derive(Default)]
#[repr(C)]
struct IoStatusBlock {
    status:      usize,
    information: usize,
}

struct Writer(usize);

static OFFSET: AtomicUsize = AtomicUsize::new(0);

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let mut li = OFFSET.fetch_add(s.len(), Ordering::Relaxed) as u64;

        let mut iosb = IoStatusBlock::default();

        unsafe {
            syscall::syscall9(
                self.0,
                0,
                0,
                0,
                &mut iosb as *mut IoStatusBlock as usize,
                s.as_ptr() as usize,
                s.len(),
                &mut li as *mut u64 as usize,
                0,
                syscall::Syscall::NtWriteFile);
        }

        Ok(())
    }
}

#[no_mangle]
extern fn __start(socket: usize) -> ! {
    let mut handle: usize = 0;

    type HANDLE = *mut u8;

    #[repr(C)]
    struct UnicodeString<'a> {
        length:     u16,
        max_length: u16,
        buffer:     &'a [u8],
    }

    #[repr(C)]
    struct ObjectAttributes<'a> {
        length:   u32,
        root:     HANDLE,
        name:     &'a UnicodeString<'a>,
        attrib:   u32,
        sec_desc: *mut u8,
        sec_qos:  *mut u8,
    }

    let mut name = UnicodeString {
        length:     0,
        max_length: 0,
        buffer:     b"\\\0D\0o\0s\0D\0e\0v\0i\0c\0e\0s\0\\\0D\0:\0\\\0n\0u\0t\0s\0.\0t\0x\0t\0"
    };
    name.length = name.buffer.len() as u16;
    name.max_length = name.buffer.len() as u16;

    let object_attributes = ObjectAttributes {
        length:   core::mem::size_of::<ObjectAttributes>() as u32,
        root:     core::ptr::null_mut(),
        name:     &name,
        attrib:   0,
        sec_desc: core::ptr::null_mut(),
        sec_qos:  core::ptr::null_mut(),
    };

    let mut iosb = IoStatusBlock {
        status:      0,
        information: 0,
    };

    unsafe {
        let ret = syscall::syscall6(
            &mut handle as *mut usize as usize,
            0x40000000,
            &object_attributes as *const ObjectAttributes as usize,
            &mut iosb as *mut IoStatusBlock as usize,
            0,
            0,
            syscall::Syscall::NtOpenFile);

        if ret != 0 {
            core::ptr::write_volatile(0x41414141 as *mut u8, 1);
        }

        macro_rules! print {
            ($($arg:tt)*) => {
                use core::fmt::Write;
                Writer(socket).write_fmt(
                    core::format_args!($($arg)*))
            }
        }

        print!("Hello world from Rust on Windows NT 4.0 MIPS {}\r\n", 69);

        for ii in 0..100 {
            print!("Apples {}\r\n", ii);
        }
    }

    syscall::exit(0);
}

