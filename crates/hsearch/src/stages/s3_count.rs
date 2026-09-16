use std::ops::BitAnd;

use super::*;

include!(concat!("../generated/stage3.rs"));

/// Stage 3: Domino misorientation
///
/// ## Invariants
///
/// - The 2x3x3x2 block of pieces at `[-1, -1, -1, 0]..[0, 1, 1, 1]` (i.e., `~(R
///   | I)`) must be setwise-preserved and remain oriented & `M`-separated.
///     - The 1x3x3x2 subblock in the `M` slice must be setwise-preserved
///     - The 1x3x3x2 subblock in the `L` layer must be setwise-preserved and
///       remain oriented.
/// - All `R`/`L` ridges must remain oriented.
///
/// ## Move set
///
/// 52 twists are allowed:
///
/// - All `R` and `I` twists (46 twists)
/// - `LO`, `LO2`, and `LI` (3 twists)
/// - `OR`, `OR2`, and `OL` (3 twists)
///
/// ## Target
///
/// - `M` ridges: 10 in `M`, 2 in `R`/`L`
/// - `R`/`L` ridges: 2 in `M`, 10 oriented in `R`/`L`
/// - `M` edges: 4 in `M`, 4 in `R`/`L`
/// - `R`/`L` edges: 4 in `M`, 4 misoriented in `R`/`L`, 4 oriented in `R`/`L`
/// - corners: 8 misoriented, 8 oriented
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Stage3 {
    /// For each `R`/`I` ridge location, 1 bit indicating one of the following
    /// cases:
    ///
    /// - `1` = belongs in `R`/`L`
    /// - `0` = belongs in `M` slice
    pub r: u16, // u11

    /// For each `R`/`I` edge location, 2 bits indicating one of the following
    /// cases:
    ///
    /// - `11` = belongs in `R`/`L`, good orientation
    /// - `01` = belongs in `R`/`L`, bad orientation 1
    /// - `10` = belongs in `R`/`L`, bad orientation 2
    /// - `00` = belongs in `M` slice, any orientation
    ///
    /// For each `R`/`I` corner location, 2 bits indicating the axis containing
    /// its `R`/`L` sticker:
    ///
    /// - `00` = X
    /// - `01` = Y
    /// - `10` = Z
    /// - `11` = W
    pub ec: u64, // (u40, u24)
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
            r: self.r & rhs.r,
            ec: self.ec & rhs.ec,
        }
    }
}

impl Stage3 {
    const fn new(r: u16, ec: u64) -> Self {
        Self { r, ec }
    }

    pub fn is_target_solved(self) -> bool {
        let Self { r, ec } = self;

        (r & Self::R_M).count_ones() == 2                         // 2 R/L ridges in M
            && (!(ec | (ec >> 1)) & Self::E_RL).count_ones() == 4 // 4 M edges in R/L
            && ((ec | (ec >> 1)) & Self::E_M).count_ones() == 4   // 4 R/L edges in M
            && ((ec ^ (ec >> 1)) & Self::E_RL).count_ones() == 4  // 4 R/L edges misoriented in R/L
            && ((ec | (ec >> 1)) & Self::C_RL).count_ones() == 8 // 8 misoriented corners
    }
}

impl Stage for Stage3 {
    fn from_state(state: SimplePuzzleSim) -> Self {
        let is_in_stage3 = |v: Vec4| v[X] == 1 || v[W] == -1;

        Self {
            r: state.pieces_to_bits(1, &[PieceType::Ridge], is_in_stage3, |init, _att| {
                (init[X] != 0) as u64
            }) as u16,
            ec: state.pieces_to_bits(
                2,
                &[PieceType::Edge, PieceType::Corner],
                is_in_stage3,
                |init, att| {
                    if init.taxicab_norm() == 3 {
                        let o = if init[X] == 0 { 0 } else { 3 };
                        hsearch_core::stage_utils::s3_eo(att, init, o) as u64 //edge
                    } else {
                        X.transform_by(att) as u64 // corner
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
    use crate::parse_twists;

    use super::*;

    #[test]
    fn test_stage3_count() {
        assert!(!Stage3::SOLVED.is_target_solved());
        assert!(Stage3::with_setup(&parse_twists("IU")).is_target_solved());
        assert!(Stage3::with_setup(&parse_twists("IF")).is_target_solved());
    }
}
