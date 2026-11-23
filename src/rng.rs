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

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_reproducible_rng() {
        let seed = 42;
        let mut rng1 = create_rng(seed);
        let mut rng2 = create_rng(seed);

        for _ in 0..10 {
            assert_eq!(rng1.gen::<u32>(), rng2.gen::<u32>());
        }
    }

    #[test]
    fn test_different_seeds_produce_different_results() {
        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(43);

        assert_ne!(rng1.gen::<u32>(), rng2.gen::<u32>());
    }

    #[test]
    fn test_rng_generates_in_valid_range() {
        let mut rng = create_rng(123);
        let val: u32 = rng.gen_range(1u32..=100u32);
        assert!(val >= 1 && val <= 100);
    }
}