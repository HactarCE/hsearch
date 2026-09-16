use std::fmt;
use std::ops::BitAnd;

use super::*;
use crate::util::collect_bits;

include!(concat!("../generated/v2_stage1.rs"));

/// Stage 1: `I`/`O` separation
///
/// ## Invariants
///
/// There are no invariants to uphold.
///
/// ## Move set
///
/// All 184 twists are allowed.
///
/// ## Targets
///
/// TBD
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(align(32))]
pub struct V2Stage1 {
    /// For each ridge sticker, 1 bit indicating whether it is an I/O sticker.
    pub r: u64, // u48 (24 ridges * 2 stickers each)
    /// For each edge sticker, 1 bit indicating whether it is an I/O sticker.
    pub e: u128, // u96 (32 edges * 3 stickers each)
    /// For each corner sticker, 1 bit indicating whether it is an I/O sticker.
    pub c: u64, // u64 (16 pieces * 4 stickers each)
}

impl Default for V2Stage1 {
    fn default() -> Self {
        Self::SOLVED
    }
}

impl fmt::Display for V2Stage1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { r, e, c } = self;
        write!(
            f,
            "V2Stage1 {{ r: 0x{r:012x}, e: 0x{e:024x}, c: 0x{c:016x} }}",
        )
    }
}

impl SubsetMaskStage for V2Stage1 {
    type Key = V2Stage1TrieKey;
}

impl V2Stage1 {
    pub fn is_target_solved(self, target: &[Self]) -> bool {
        target.iter().any(|&t| self & t == t)
    }

    fn stickers(piece_type: PieceType) -> impl Iterator<Item = Vec4> {
        Facet::ALL
            .iter()
            .flat_map(|&f| {
                let mut min = Vec4([-1; 4]);
                let mut max = Vec4([1; 4]);
                min[f.axis()] = f.sign() as i8 * 2;
                max[f.axis()] = f.sign() as i8 * 2;
                Vec4::region(min, max)
            })
            .filter(move |v| v.taxicab_norm() - 1 == piece_type.sticker_count())
    }
    fn ridge_stickers() -> impl Iterator<Item = Vec4> {
        Self::stickers(PieceType::Ridge)
    }
    fn edge_stickers() -> impl Iterator<Item = Vec4> {
        Self::stickers(PieceType::Edge)
    }
    fn corner_stickers() -> impl Iterator<Item = Vec4> {
        Self::stickers(PieceType::Corner)
    }
}

impl BitAnd for V2Stage1 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r & rhs.r,
            e: self.e & rhs.e,
            c: self.c & rhs.c,
        }
    }
}

impl Stage for V2Stage1 {
    fn from_state(state: SimplePuzzleSim) -> Self {
        let is_w_sticker = |sticker_vector: Vec4| {
            let (pos, ax) = sticker_vector.unwrap_sticker();
            let (_init, att) = state.get_piece(pos);
            att[ax][W] != 0
        };
        Self {
            r: collect_bits(Self::ridge_stickers().map(is_w_sticker)),
            e: collect_bits(Self::edge_stickers().map(is_w_sticker)),
            c: collect_bits(Self::corner_stickers().map(is_w_sticker)),
        }
    }

    fn do_twist(self, twist: Twist) -> Self {
        self.generated_do_twist(twist)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_type_stickers() {
        assert_eq!(48, V2Stage1::ridge_stickers().count());
        assert_eq!(96, V2Stage1::edge_stickers().count());
        assert_eq!(64, V2Stage1::corner_stickers().count());
    }
}
