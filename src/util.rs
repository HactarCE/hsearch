use itertools::Itertools;
use rand::SeedableRng;
use rand::seq::IteratorRandom;

use crate::{Twist, TwistData};

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

/// Collects up to 64 booleans into a bitmask.
#[cfg(test)]
pub fn collect_bits(iter: impl IntoIterator<Item = bool>) -> u64 {
    iter.into_iter()
        .take(64)
        .positions(|b| b)
        .map(|i| 1 << i)
        .fold(0, |a, b| a | b)
}

/// Parses twists from a string.
///
/// # Panics
///
/// Panics if a twist is invalid.
pub fn parse_twists(s: &str) -> Vec<TwistData> {
    s.split_ascii_whitespace()
        .map(|word| TwistData::from_notation(word).expect("invalid twist"))
        .collect()
}

/// Serializes twists to an HSC2-compatible string.
pub fn twists_to_string(twists: &[Twist]) -> String {
    twists
        .iter()
        .map(|t| t.data().to_string())
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
    std::iter::from_fn(|| Twist::iter().choose(&mut rng))
        .take(crate::SCRAMBLE_LEN)
        .collect()
}
