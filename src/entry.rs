//! Main program entry point

use crate::syscall::{exit, Handle};

/// Exported entry point
#[no_mangle]
pub unsafe extern fn __start(socket: Handle) -> ! {
    // Register the socket with the print handler so that we can print!
    crate::print::register_socket(socket);

    // Invoke main
    if let Err(x) = crate::main() {
        panic!("main exited with error: {:?}", x);
    }

    // Exit the program
    exit(0);
}

