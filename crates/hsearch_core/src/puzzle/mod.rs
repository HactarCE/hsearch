mod piece_type;
mod sim;
mod twist;
mod twist_data;
mod twist_names;
mod twist_set;

pub use crate::linalg::Facet;
pub use piece_type::PieceType;
pub use sim::SimplePuzzleSim;
pub use twist::Twist;
pub use twist_data::TwistData;
pub use twist_set::TwistSet;

/// Number of unique twists on the puzzle.
pub const TWIST_COUNT: u8 = 8 * 23;

/// Returns an iterator over all the stickers on the puzzle.
pub fn all_stickers() -> impl Iterator<Item = crate::Vec4> {
    Facet::ALL.iter().flat_map(|f| f.stickers())
}

/// Parses twists from a string.
///
/// # Panics
///
/// Panics if a twist is invalid.
pub fn parse_twists(s: &str) -> Vec<Twist> {
    s.split_ascii_whitespace()
        .filter(|&word| word != ".")
        .map(|word| Twist::from_name(word).unwrap_or_else(|| panic!("invalid twist {word:?}")))
        .collect()
}

/// Serializes twists to an HSC2-compatible string.
pub fn twists_to_string(twists: &[Twist]) -> String {
    use itertools::Itertools;

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
