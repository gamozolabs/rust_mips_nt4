//! Xorshift64

use core::cell::Cell;

/// Xorshift64 implementation
pub struct Rng(Cell<u64>);

impl Rng {
    /// Create a seeded RNG
    pub const fn new(seed: u64) -> Self {
        Self(Cell::new(seed))
    }

    /// Get the next RNG value
    pub fn next(&self) -> u64 {
        let mut seed = self.0.get();
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 43;
        self.0.set(seed);
        seed
    }
}

