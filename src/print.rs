//! Print and debug routines

use core::sync::atomic::{AtomicUsize, Ordering};
use crate::syscall::Handle;

/// Classic `print!()` macro
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let _ = core::fmt::Write::write_fmt(
            &mut $crate::print::Writer(
                crate::syscall::Handle(crate::print::SOCKET.load(
                    core::sync::atomic::Ordering::Relaxed))),
                core::format_args!($($arg)*));
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

/// Socket for external print communication
pub static SOCKET: AtomicUsize = AtomicUsize::new(0);

/// Writer structure that simply implements [`core::fmt::Write`] such that we
/// can use `write_fmt` in our [`print!`]
pub struct Writer(pub Handle);

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let _ = crate::syscall::write(self.0, s);

        Ok(())
    }
}

/// Register the initial socket
pub(super) unsafe fn register_socket(socket: Handle) {
    SOCKET.store(socket.0, Ordering::Release);
}

