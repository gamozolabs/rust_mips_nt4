//! Main program entry point for Rust

#![feature(asm, naked_functions, asm_experimental_arch)]
#![feature(default_alloc_error_handler, new_uninit, asm_const)]
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
    let asdf = 5;

    for _ in 0..10000 {
        let thing = syscall::spawn(move || {
            println!("asdgf {}", asdf);
            core::cell::RefCell::new(1234)
        }).unwrap();
        dbg!(thing.join().unwrap());
    }

    Ok(())
}

