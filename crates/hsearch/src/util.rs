use itertools::Itertools;
use rand::SeedableRng;
use rand::seq::IteratorRandom;

use crate::{Twist, TwistData};

/// Parses twists from a string.
///
/// # Panics
///
/// Panics if a twist is invalid.
pub fn parse_twists(s: &str) -> Vec<Twist> {
    s.split_ascii_whitespace()
        .filter(|&word| word != ".")
        .map(|word| {
            TwistData::from_notation(word).unwrap_or_else(|| panic!("invalid twist {word:?}"))
        })
        .map(|data| crate::TWIST_DATA_TO_TWIST[&data])
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
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    std::iter::from_fn(|| Twist::iter().choose(&mut rng))
        .take(crate::SCRAMBLE_LEN)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twists_round_trip() {
        for t in Twist::iter() {
            assert_eq!(vec![t], parse_twists(&t.data().to_string()))
        }
    }
}
