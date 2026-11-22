//! Random number generation utilities.
//!
//! This module provides a seeded random number generator to ensure
//! that data generation is fully reproducible given the same seed.

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Creates a new ChaCha8-based random number generator seeded with the provided value.
///
/// # Arguments
///
/// * `seed` - A 64-bit unsigned integer used to initialize the RNG state.
///
/// # Returns
///
/// A `ChaCha8Rng` instance configured with the given seed.
pub fn create_rng(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}
