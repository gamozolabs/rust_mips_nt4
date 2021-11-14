//! Print and debug routines

use crate::syscall::Handle;

/// Classic `print!()` macro
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let _ = core::fmt::Write::write_fmt(
            &mut $crate::print::Writer, core::format_args!($($arg)*));
    }
}

/// Classic `println!()` macro
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        print!($($arg)*);
        print!("\r\n");
    }
}

/// Classic `dbg!()` macro
#[macro_export]
macro_rules! dbg {
    // NOTE: We cannot use `concat!` to make a static string as a format
    // argument of `println!` because `file!` could contain a `{` or `$val`
    // expression could be a block (`{ .. }`), in which case the `println!`
    // will be malformed.
    () => {
        $crate::println!("[{}:{}]", file!(), line!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::println!("[{}:{}] {} = {:#?}",
                    file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

/// Writer structure that simply implements [`core::fmt::Write`] such that we
/// can use `write_fmt` in our [`print!`]
pub struct Writer;

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let _ = crate::syscall::write(unsafe { &SOCKET }, s);

        Ok(())
    }
}

/// The socket handle
static mut SOCKET: Handle = unsafe { Handle::from_raw(0) };

/// Register the initial socket
pub(super) unsafe fn register_socket(socket: Handle) {
    core::ptr::write(&mut SOCKET, socket);
}

