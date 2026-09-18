use rand::SeedableRng;
use rand::seq::IteratorRandom;

use crate::Twist;

/// Returns a deterministic scramble from a random seed.
pub fn scramble(seed: u64) -> Vec<Twist> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    std::iter::from_fn(|| Twist::iter().choose(&mut rng))
        .take(crate::SCRAMBLE_LEN)
        .collect()
}
