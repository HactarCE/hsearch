use crate::XyRot;

use super::*;

include!(concat!(env!("OUT_DIR"), "/stage4.rs"));

/// Stage 4: `P`-separation + `I`/`O` edge & corner orientation
///
/// ## Invariants
///
/// - All `I`/`O` ridges must remain oriented.
/// - All pieces outside the `F` facet (`[-1, -1, 1, -1]..=[1, 1, 1, 1]`) and
///   `UO` (`[-1, -1, 1, 1]..=[1, 1, 1, 1]`) ridge must remain oriented &
///   `P`-separated.
///
/// ## Move set
///
/// 33 twists are allowed:
///
/// - All `F` twists (23 twists)
/// - `IF`, `IF2`, and `IB` (3 twists)
/// - `OF`, `OF2`, and `OB` (3 twists)
/// - Certain `O` twists, followed by an implicit rotation to preserve
///   invariants:
///     - `OR` (1 twist)
///     - `OUFR`, and `OUFL` (2 twists)
///     - `OUF` (1 twist)
///
/// ## Target
///
/// - All `P` pieces are in `P`.
/// - All `I`/`O` pieces are in `I`/`O` and are `I`/`O`-oriented.
///
/// This target has 1 possible orientation.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Stage4 {
    /// For each OU/F corner, 2 bits indicating the axis containing its I/O
    /// sticker:
    ///
    /// - `00` = X
    /// - `01` = Y
    /// - `10` = Z
    /// - `11` = W
    ///
    /// For each OU/F edge, 2 bits indicating one of the following cases:
    ///
    /// - `00` = belongs in P slice, any orientation
    /// - `01` = belongs in I/O, good orientation
    /// - `10` = belongs in I/O, bad orientation 1
    /// - `11` = belongs in I/O, bad orientation 2
    ///
    /// For each OU/F ridge, 1 bit indicating one of the following cases:
    ///
    /// - `0` = belongs in the P slice
    /// - `1` = belongs in I/O
    ///
    /// Total: 6 corners (20 bits), 15 edges (30 bits), 7 ridges (7 bits)
    pub bits: u64, // (u20, u30, u7)
}

impl Default for Stage4 {
    fn default() -> Self {
        Self::SOLVED
    }
}

impl Stage4 {
    pub fn is_target_solved(self, target: Self) -> bool {
        self.bits == target.bits
    }
}

impl StageKeyU64 for Stage4 {
    fn key(self) -> u64 {
        self.bits
    }
    const PRUNING_MAP_TWISTS: &[Twist] = &Self::TWISTS;
}

impl Stage for Stage4 {
    fn implicit_rotation_after_twist(twist: Twist) -> XyRot {
        Self::generated_implicit_rotation_after_twist(twist)
    }

    fn do_twist(self, twist: Twist) -> Self {
        self.generated_do_twist(twist)
    }

    fn from_state(state: SimplePuzzleSim) -> Self {
        for &(pos, att) in &state.pieces {
            let p = att * pos;
            if !((p[W] == 1 && p[Y] == 1) || p[Z] == 1) && p.taxicab_norm() >= 3 {
                assert_eq!(W.transform_by(att), W, "pos is bad: {}", p);
            }
        }
        Self {
            bits: state.to_bits(
                2,
                &[PieceType::Corner, PieceType::Edge],
                |v| (v[W] == 1 && v[Y] == 1) || v[Z] == 1,
                |init, att| {
                    if init.taxicab_norm() == 3 {
                        // edge
                        if init[W] == 0 {
                            0
                        } else {
                            let old_io_sticker_axis = W;
                            let new_io_sticker_axis = old_io_sticker_axis.transform_by(att);
                            let new_index = (att * init)
                                .nonzero_axes()
                                .iter()
                                .position(|&ax| ax == new_io_sticker_axis)
                                .unwrap();
                            3 - new_index as u64
                        }
                    } else {
                        // corner
                        W.transform_by(att) as u64
                    }
                },
            ) | (state.to_bits(
                1,
                &[PieceType::Ridge],
                |v| (v[W] == 1 && v[Y] == 1) || v[Z] == 1,
                |init, _att| (init[W] != 0) as u64,
            ) << 50),
        }
    }
}
