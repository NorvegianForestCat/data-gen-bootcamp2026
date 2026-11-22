use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

pub fn create_rng(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}