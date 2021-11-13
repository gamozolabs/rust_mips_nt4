//! Panic handler

/// Panic handler
#[panic_handler]
fn panic_handler(pi: &core::panic::PanicInfo) -> ! {
    println!("{}", pi);
    crate::syscall::exit(!0);
}

