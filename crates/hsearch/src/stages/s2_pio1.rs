use std::ops::BitAnd;

use super::*;

include!(concat!(env!("OUT_DIR"), "/stage2.rs"));

/// Stage 2: partial `I`/`O` edge & corner orientation (3x3x2x1 block)
///
/// ## Invariants
///
/// - All `I`/`O` ridges must remain oriented.
/// - The 3x3x2x1 block of `P` pieces in `P` at `[-1, -1, -1, 0]..=[1, 1, 0, 0]`
///   (i.e., `~(F | O | I)`) must be setwise-preserved.
///
/// ## Move set
///
/// 80 twists are allowed:
///
/// - All `F`, `I`, and `O` twists (69 twists)
/// - `RF2`, `LF2`, `UF2`, and `DF2` (4 twists)
/// - `BO`, `BO2`, and `BI` (3 twists)
/// - `BR2`, `BU2`, `BUR`, and `BUL` (4 twists)
///
/// ## Targets
///
/// ### Target 1
///
/// - 2x2x2x1 block of `I`/`O`-oriented pieces in `I` (`[-1, -1, -1, -1]..=[0,
///   0, 0, -1]`)
///     - 3 ridges (already oriented from stage 1)
///     - 3 oriented `I`/`O` edges
///     - 1 oriented corner
///
/// This target has 16 possible orientations.
///
/// ### Target 2
///
/// - 3x2x2x1 block of `I`/`O`-oriented pieces in `I` (`[-1, -1, -1, -1]..=[1,
///   0, 0, -1]`)
///     - 4 ridges (already oriented from stage 1)
///     - 5 oriented `I`/`O` edges
///     - 2 oriented corners
///
/// This target has 24 possible orientations.
///
/// ### Target 3
///
/// - 3x3x2x1 block of `I`/`O`-oriented pieces in `I` (`[-1, -1, -1, -1]..=[1,
///   1, 0, -1]`)
///     - 5 ridges (already oriented from stage 1)
///     - 8 oriented `I`/`O` edges
///     - 4 oriented corners
///
/// This target has 2 possible orientations relative to the invariant block on
/// `P`: the new block may be on `I` or it may be on `O`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Stage2 {
    /// For each F/O/I ridge, 1 bit indicating one of the following cases:
    ///
    /// - `0` = belongs in the P slice
    /// - `1` = belongs in I/O
    pub r_p: u16, // u16

    /// For each F/O/I edge, 2 bits indicating one of the following cases:
    ///
    /// - `00` = belongs in P slice, any orientation
    /// - `01` = belongs in I/O, good orientation
    /// - `10` = belongs in I/O, bad orientation 1
    /// - `11` = belongs in I/O, bad orientation 2
    pub e_op: u64, // u56

    /// For each F/O/I corner, 2 bits indicating the axis containing its I/O
    /// sticker:
    ///
    /// - `00` = X
    /// - `01` = Y
    /// - `10` = Z
    /// - `11` = W
    pub c_o: u32, // u32
}

impl Default for Stage2 {
    fn default() -> Self {
        Self::SOLVED
    }
}

impl BitAnd for Stage2 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            r_p: self.r_p & rhs.r_p,
            e_op: self.e_op & rhs.e_op,
            c_o: self.c_o & rhs.c_o,
        }
    }
}

impl Stage2 {
    const fn new(r_p: u16, e_op: u64, c_o: u32) -> Self {
        Self { r_p, e_op, c_o }
    }

    pub fn is_target_solved(self, target: &[Self]) -> bool {
        target.iter().any(|&t| self & t == Self::SOLVED & t)
    }

    /// Returns the sign of the unsolved facet.
    pub fn which_target3(self) -> Option<Sign> {
        Self::TARGET3
            .iter()
            .position(|&t| self.is_target_solved(&[t]))
            .map(|i| [Sign::Pos, Sign::Neg][i])
    }
}

impl Stage for Stage2 {
    fn from_state(state: SimplePuzzleSim) -> Self {
        Self {
            r_p: state.to_bits(
                1,
                &[PieceType::Ridge],
                |v| v[W] != 0 || v[Z] == 1,
                |init, _att| (init[W] != 0) as u64,
            ) as u16,
            e_op: state.to_bits(
                2,
                &[PieceType::Edge],
                |v| v[W] != 0 || v[Z] == 1,
                |init, att| {
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
                },
            ),
            c_o: state.to_bits(
                2,
                &[PieceType::Corner],
                |_| true,
                |_init, att| W.transform_by(att) as u64,
            ) as u32,
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
    fn test_stage2_which_target3() {
        assert_eq!(
            Some(Sign::Neg),
            Stage2::with_setup(&crate::parse_twists("FD ID FU ID FD ID FU")).which_target3(),
        );
        assert_eq!(
            Some(Sign::Pos),
            Stage2::with_setup(&crate::parse_twists("FD OD FU OD FD OD FU")).which_target3(),
        );
    }
}
