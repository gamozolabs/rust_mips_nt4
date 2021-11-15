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
mod rand;
mod panic;
mod syscall;

use alloc::vec::Vec;
use crate::rand::Rng;

/// Worker thread for fuzzing
fn worker(id: usize) {
    // Create an RNG
    let rng = Rng::new(0xe06fc2cdf7b80594 + id as u64);

    loop {
        unsafe {
            syscall::syscall9(
                rng.next() as usize,
                rng.next() as usize,
                rng.next() as usize,
                rng.next() as usize,
                rng.next() as usize,
                rng.next() as usize,
                rng.next() as usize,
                rng.next() as usize,
                rng.next() as usize,
                rng.next() as usize);
        }
    }
}

/// Run the fuzzer on multiple threads!
fn fuzz() {
    // Thread handlers for workers
    let mut workers = Vec::new();

    // Spawn worker threads
    for ii in 0..8 {
        workers.push(
            syscall::spawn(move || worker(ii)).expect("Failed to spawn thread")
        );
    }

    // Wait for all threads to exit
    for thr in workers {
        thr.join().expect("Failed to join thread");
    }
}

fn main() -> Result<(), ()> {
    fuzz();
    Ok(())
}

