use std::{collections::BTreeMap, sync::LazyLock};

use itertools::Itertools;

use super::*;

include!(concat!("../generated/stage4.rs"));

/// Stage 4: Pre-domino
///
/// ## Projection
///
/// This stage uses the domino projection described in the [crate] docs.
///
/// ## Invariants
///
/// - The number and types of domino-(mis)oriented pieces must remain constant
///   from stage 3.
///
/// ## Move set
///
/// 88 twists are allowed:
///
/// - All `R` and `L` twists (46 twists)
/// - `U`, `D`, `F`, `B`, `O`, `I` twists that stabilize the `X` axis (42
///   twists)
///
/// ## Target
///
/// - The puzzle is one (non-domino) move away from a valid domino-reduced
///   state.
///
/// This target has 12 possible states under the projection, all in the same
/// orbit under domino symmetry.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Stage4 {
    /// For each ridge location, 2 bits indicating one of the following cases:
    ///
    /// - `00` = belongs in `M`
    /// - `01` = belongs in `R`/`L`, orientation 1
    /// - `10` = belongs in `R`/`L`, orientation 2
    r: u64, // u48

    /// For each edge location, 2 bits indicating one of the following cases:
    ///
    /// - `11` = belongs in `R`/`L`, good orientation
    /// - `01` = belongs in `R`/`L`, bad orientation 1
    /// - `10` = belongs in `R`/`L`, bad orientation 2
    /// - `00` = belongs in `M` slice, any orientation
    e: u64, // u64

    /// For each corner location, 2 bits indicating the axis containing its
    /// `R`/`L` sticker:
    ///
    /// - `00` = X
    /// - `01` = Y
    /// - `10` = Z
    /// - `11` = W
    c: u32, // u32
}

impl Default for Stage4 {
    fn default() -> Self {
        Self::SOLVED
    }
}

impl Stage4 {
    // pub fn is_322_target_solved(self) -> bool {}

    // pub fn is_321_target_solved(self) -> bool {}

    pub fn is_311_target_solved(self, target_block_count: u8) -> bool {
        type S4TargetBlocks = [u128; 18];

        /// Returns a list of 1x1x1x3 blocks, given an axis order.
        ///
        /// - `axes[0]` is the axis along which only ±1 are considered (not 0)
        /// - `axes[1]` and `axes[2]` are the axes along which 0 and ±1 are
        ///   considered.
        /// - `axes[3]` is the axis along which the `3` dimension of the block
        ///   is oriented
        fn blocks_3111(axes: [Axis; 4]) -> [u128; 18] {
            let [a, b, c, _d] = axes;
            itertools::iproduct!([-1, 1], -1..=1, -1..=1)
                .map(|(q, r, s)| Stage4::packed_piece_mask(|v| v[a] == q && v[b] == r && v[c] == s))
                .collect_array()
                .unwrap()
        }

        // We are not actually targeting these states per se because they have
        // too many pieces unsolved, but these states are dense with pieces in
        // the desired orientation for a corresponding target state.
        static TARGET_STATES: LazyLock<[Stage4; 6]> = LazyLock::new(|| {
            [
                Stage4::with_setup(&parse_twists("UF DF")),
                Stage4::with_setup(&parse_twists("UO DO")),
                Stage4::with_setup(&parse_twists("FU BU")),
                Stage4::with_setup(&parse_twists("FO BO")),
                Stage4::with_setup(&parse_twists("OU IU")),
                Stage4::with_setup(&parse_twists("OF IF")),
            ]
        });

        // 3x1x1 blocks not along the X axis
        static TARGET_BLOCKS: LazyLock<[S4TargetBlocks; 6]> = LazyLock::new(|| {
            [
                blocks_3111([Y, X, Z, W]),
                blocks_3111([Y, X, W, Z]),
                blocks_3111([Z, X, Y, W]),
                blocks_3111([Z, X, W, Y]),
                blocks_3111([W, X, Y, Z]),
                blocks_3111([W, X, Z, Y]),
            ]
        });

        static TARGET_BLOCK_MASKS: LazyLock<(Vec<u128>, Vec<u8>)> = LazyLock::new(|| {
            let mut block_to_states = BTreeMap::new(); // btreemap for determinism
            for (i, blocks) in TARGET_BLOCKS.iter().enumerate() {
                for &block in blocks {
                    *block_to_states.entry(block).or_default() |= 1 << i;
                }
            }
            (
                block_to_states.keys().copied().collect(),
                block_to_states.values().copied().collect(),
            )
        });

        let mut similar_piece_masks = TARGET_STATES.map(|target| self.similar_piece_mask(target));

        let mut block_count = 0;
        let (target_block_masks, states_for_target_block_masks) = &*TARGET_BLOCK_MASKS;
        for (&target_block_mask, &target_block_state_mask) in
            std::iter::zip(target_block_masks, states_for_target_block_masks)
        {
            if similar_piece_masks
                .iter()
                .enumerate()
                .any(|(i, similar_pieces)| {
                    target_block_state_mask & (1 << i) != 0
                        && similar_pieces & target_block_mask == target_block_mask
                })
            {
                block_count += 1;
                if block_count >= target_block_count {
                    return true;
                }
                for similar_pieces in &mut similar_piece_masks {
                    *similar_pieces &= !target_block_mask
                }
            }
        }

        false
    }

    /// Returns a bitmask of pieces that are the same between `self` and
    /// `target`.
    ///
    /// The pieces are stored at the following locations:
    ///
    /// - `0x5555_5555_5555_0000_0000` = ridges
    /// - `0x0000_aaaa_aaaa_aaaa_aaaa` = edges
    /// - `0x0000_0000_0000_5555_5555` = corners
    fn similar_piece_mask(self, target: Self) -> u128 {
        let r = self.r ^ target.r; // u48
        let e = self.e ^ target.e; // u64
        let c = self.c ^ target.c; // u32
        // reduce to 1 bit per piece, using only even bit indices
        let r = (r | (r >> 1)) & 0x5555_5555_5555;
        let e = (e | (e >> 1)) & 0x5555_5555_5555_5555;
        let c = (c | (c >> 1)) & 0x5555_5555;
        let even_bits = ((r as u128) << 32) | c as u128;
        let odd_bits = (e << 1) as u128;
        (even_bits | odd_bits) ^ 0x5555_ffff_ffff_ffff_ffff
    }

    fn packed_piece_mask(f: impl Fn(Vec4) -> bool) -> u128 {
        let r = PieceType::Ridge.iter().map(&f);
        let e = PieceType::Edge.iter().map(&f);
        let c = PieceType::Corner.iter().map(&f);
        let even_bits = c.chain(r);
        let odd_bits = e.chain([false; 8]);
        debug_assert_eq!(even_bits.clone().count(), 40);
        debug_assert_eq!(odd_bits.clone().count(), 40);
        collect_bits(even_bits.interleave(odd_bits))
    }

    /// Returns the set of target states.
    pub fn target() -> Vec<Stage4> {
        // more moves are possible, but these are enough to cover all 12 unique
        // states under domino projection
        parse_twists("UF UO DF DO FU FO BU BO OU OF IU IF")
            .into_iter()
            .map(|twist| Self::with_setup(&[twist]))
            .collect()
    }

    pub fn is_target_solved(self, targets: &[Self]) -> bool {
        targets.contains(&self)
    }
}

impl StageKeyU128 for Stage4 {
    fn init() -> Vec<Self> {
        Self::target()
    }
    fn key(self) -> u128 {
        let r = if self.r == 0 {
            0
        } else {
            u32::from_ne_bytes(
                bit_iter::BitIter::from(self.r ^ Self::SOLVED.r)
                    .collect_array()
                    .unwrap()
                    .map(|i| i as u8),
            )
        };

        self.e as u128 | ((self.c as u128) << 64) | ((r as u128) << (64 + 32))
    }
    const PRUNING_MAP_TWISTS: TwistSet = Self::TWISTS;
}

impl Stage for Stage4 {
    const TWISTS: TwistSet = Self::GENERATED_TWISTS;

    fn do_twist(self, twist: Twist) -> Self {
        self.generated_do_twist(twist)
    }

    fn from_state(state: SimplePuzzleSim) -> Self {
        let r = state.pieces_to_bits(
            2,
            &[PieceType::Ridge],
            |_| true,
            |init, att| hsearch_core::stage_utils::xyz_ro(att, init, (init[X] != 0) as u8) as u64,
        );

        let e = state.pieces_to_bits(
            2,
            &[PieceType::Edge],
            |_| true,
            hsearch_core::stage_utils::rl_init_eo,
        );

        let c = state.pieces_to_bits(
            2,
            &[PieceType::Corner],
            |_| true,
            |_init, att| X.transform_by(att) as u64,
        ) as u32;

        Self { r, e, c }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage4_packed_bits() {
        for piece_type in [PieceType::Ridge, PieceType::Edge, PieceType::Corner] {
            let mask =
                Stage4::packed_piece_mask(|v| v.taxicab_norm() == piece_type.sticker_count());
            assert_eq!(mask.count_ones() as usize, piece_type.iter().count());
        }

        let zero = Stage4 { r: 0, e: 0, c: 0 };
        let full = Stage4 {
            r: (1 << 48) - 1,
            e: u64::MAX,
            c: u32::MAX,
        };

        // Check ridge mask
        let s = Stage4 { r: 0, ..full };
        let diff1 = s.similar_piece_mask(zero);
        let diff2 = zero.similar_piece_mask(s);
        assert_eq!(diff1, diff2);
        assert_eq!(diff1, 0x5555_5555_5555_0000_0000);

        // Check edge mask
        let s = Stage4 { e: 0, ..full };
        let diff1 = s.similar_piece_mask(zero);
        let diff2 = zero.similar_piece_mask(s);
        assert_eq!(diff1, diff2);
        assert_eq!(diff1, 0x0000_aaaa_aaaa_aaaa_aaaa);

        // Check corner mask
        let s = Stage4 { c: 0, ..full };
        let diff1 = s.similar_piece_mask(zero);
        let diff2 = zero.similar_piece_mask(s);
        assert_eq!(diff1, diff2);
        assert_eq!(diff1, 0x0000_0000_0000_5555_5555);
    }

    #[test]
    fn test_partial_target_no_panic() {
        Stage4::SOLVED.is_311_target_solved(0);
    }
}
