//! Main program entry point for Rust

#![feature(asm, naked_functions, asm_experimental_arch)]
#![feature(default_alloc_error_handler)]
#![no_std]
#![no_main]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

#[macro_use] pub mod print;
mod entry;
mod mman;
mod panic;
mod syscall;

fn main() -> Result<(), ()> {
    let veccy = vec![0u8; 1024 * 1024];
    let format = format!("moose {:p}\n", veccy.as_ptr());

    println!("{}", format);

    Ok(())
}

