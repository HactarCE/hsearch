use itertools::Itertools;
use rand::{SeedableRng, seq::IndexedRandom};

use crate::Twist;

/// Returns an iterator over permutations of a list, each with whether it is
/// odd.
pub fn permutations_with_parity<I>(iter: I) -> impl Iterator<Item = (Vec<I::Item>, bool)>
where
    I: ExactSizeIterator,
    I::Item: Clone,
{
    let len = iter.len();
    iter.permutations(len)
        .enumerate()
        .map(|(i, p)| (p, is_permutation_odd(i)))
}

/// Returns the parity of the permutation with number `n`.
pub fn is_permutation_odd(mut n: usize) -> bool {
    let mut res = false;
    let mut i = 2;
    while n > 0 {
        res ^= !(n % i).is_multiple_of(2);
        n /= i;
        i += 1;
    }
    res
}

/// Parses twists from a string.
///
/// # Panics
///
/// Panics if a twist is invalid.
pub fn parse_twists(s: &str) -> Vec<Twist> {
    s.split_ascii_whitespace()
        .map(|word| Twist::from_notation(word).expect("invalid twist"))
        .collect()
}

/// Serializes twists to an HSC2-compatible string.
pub fn twists_to_string(twists: &[Twist]) -> String {
    twists
        .iter()
        .map(|t| t.to_string())
        .map(|s| {
            // work around a twist parsing bug in HSC2<=2.0.0-zeta.12
            s.strip_suffix('2')
                .map(|fam| format!("{fam} {fam}"))
                .unwrap_or(s)
        })
        .join(" ")
}

/// Returns a deterministic scramble from a random seed.
pub fn scramble(seed: u64) -> Vec<Twist> {
    let mut seed_bytes = [0; 32];
    seed_bytes[..8].copy_from_slice(&seed.to_le_bytes());
    let mut rng = rand::rngs::StdRng::from_seed(seed_bytes);
    crate::HYPERCUBE_TWISTS
        .choose_iter(&mut rng)
        .expect("error generating scramble")
        .copied()
        .take(crate::SCRAMBLE_LEN)
        .collect()
}
