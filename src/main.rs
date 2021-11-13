//! Main program entry point for Rust

#![feature(asm, naked_functions, asm_experimental_arch)]
#![no_std]
#![no_main]

#[macro_use] pub mod print;
mod entry;
mod panic;
mod syscall;

fn main() -> Result<(), ()> {
    println!("Hello world");
    Ok(())
}

