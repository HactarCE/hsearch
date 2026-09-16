use std::ops::BitAnd;

use super::*;

include!(concat!(env!("OUT_DIR"), "/stage3.rs"));

/// Stage 3: partial `I`/`O` edge & corner orientation (3x2x2x1 block)
///
/// ## Invariants
///
/// - All `I`/`O` ridges must remain oriented.
/// - The 3x3x2x2 block of pieces at `[-1, -1, -1, -1]..=[1, 1, 0, 0]` (i.e.,
///   `~(F | O)`) must be setwise-preserved and remain oriented & `P`-separated.
///     - The 3x3x2x1 subblock in the `P` slice must be setwise-preserved
///     - The 3x3x2x1 subblock in the `I` layer must be setwise-preserved and
///       remain oriented.
///
/// ## Move set
///
/// 46 twists are allowed:
///
/// - All `F` and `O` twists
///
/// ## Targets
///
/// ### Target 1
///
/// - 2x2x2x1 block of `I`/`O`-oriented pieces in `O` (`[-1, -1, -1, 1]..=[0, 0,
///   0, 1]`)
///     - 3 ridges (already oriented from stage 1)
///     - 3 oriented `I`/`O` edges
///     - 1 oriented corner
///
/// This target has 4 possible orientations.
///
/// ### Target 2
///
/// - 3x2x2x1 block of `I`/`O`-oriented pieces in `O` (`[-1, -1, -1, 1]..=[1, 0,
///   0, 1]`)
///     - 4 ridges (already oriented from stage 1)
///     - 5 oriented `I`/`O` edges
///     - 2 oriented corners
///
/// This target has 4 possible orientations relative to the invariant block on
/// `P`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Stage3 {
    /// For each F/O ridge, 1 bit indicating one of the following cases:
    ///
    /// - `0` = belongs in the P slice
    /// - `1` = belongs in I/O
    pub r_p: u16, // u11

    /// For each F/O edge, 2 bits indicating one of the following cases:
    ///
    /// - `00` = belongs in P slice, any orientation
    /// - `01` = belongs in I/O, good orientation
    /// - `10` = belongs in I/O, bad orientation 1
    /// - `11` = belongs in I/O, bad orientation 2
    ///
    /// For each F/O corner, 2 bits indicating the axis containing its I/O
    /// sticker:
    ///
    /// - `00` = X
    /// - `01` = Y
    /// - `10` = Z
    /// - `11` = W
    pub e_op_c_o: u64, // (u40, u24)
}

impl Default for Stage3 {
    fn default() -> Self {
        Self::SOLVED
    }
}

impl BitAnd for Stage3 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            r_p: self.r_p & rhs.r_p,
            e_op_c_o: self.e_op_c_o & rhs.e_op_c_o,
        }
    }
}

impl Stage3 {
    const fn new(r_p: u16, e_op_c_o: u64) -> Self {
        Self { r_p, e_op_c_o }
    }

    pub fn is_target_solved(self, target: &[Self]) -> bool {
        target.iter().any(|&t| self & t == Self::SOLVED & t)
    }

    pub fn which_target2(self) -> Option<Facet> {
        Self::TARGET2
            .iter()
            .position(|&t| self.is_target_solved(&[t]))
            .map(|i| [Facet::U, Facet::R, Facet::L, Facet::D][i])
    }
}

impl Stage for Stage3 {
    fn from_state(state: SimplePuzzleSim) -> Self {
        Self {
            r_p: state.to_bits(
                1,
                &[PieceType::Ridge],
                |v| v[W] == 1 || v[Z] == 1,
                |init, _att| (init[W] != 0) as u64,
            ) as u16,
            e_op_c_o: state.to_bits(
                2,
                &[PieceType::Edge, PieceType::Corner],
                |v| v[W] == 1 || v[Z] == 1,
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
            ),
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
    fn test_stage3_which_target2() {
        for (scramble, target) in [
            ("UL FR UR FL", Facet::U),
            ("RU FD RD FU", Facet::R),
            ("DR FL DL FR", Facet::D),
            ("LD FU LU FD", Facet::L),
        ] {
            assert_eq!(
                Some(target),
                Stage3::with_setup(&crate::parse_twists(scramble)).which_target2()
            );
        }
    }
}
