use std::fmt;

use itertools::Itertools;

use super::*;

include!(concat!("../generated/stage4.rs"));

/// Stage 4: Pre-domino
///
/// ## Invariants
///
/// - All pieces must maintain their orientation with respect to the `X` axis.
///
/// ## Move set
///
/// 102 twists are allowed:
///
/// - All `R` and `L` twists (46 twists)
/// - `U`, `D`, `F`, `B`, `O`, `I` twists that stabilize the `X` axis (56 twists)
///
/// ## Target
///
/// - The puzzle is one move away from domino reduction.
///
/// This target has 12 possible orientations.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Stage4 {
    /// For each of the two `R`/`L` ridge pieces in `M`:
    ///
    /// - 3 bits indicating the facet containing the `R`/`L` sticker
    /// - 3 bits indicating the facet containing the non-`R`/`L` sticker
    ///
    /// For each of the two `M` ridge pieces in `R`/`L`:
    ///
    /// - 3 bits indicating the `R`/`L` facet the ridge is on
    /// - 3 bits indicating the non-`R`/`L` facet the ridge is on
    ///
    /// Each pair of ridges is kept sorted by bit pattern to canonicalize the
    /// overall bit pattern.
    r: [RidgePos; 4], // u24

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
    /// Returns the set of target states.
    pub fn target() -> Vec<Stage4> {
        parse_twists("UF UO DF DO FU FO BU BO OU OF IU IF")
            .into_iter()
            .map(|twist| Self::with_setup(&[twist]))
            .collect()
    }

    pub fn is_target_solved(self, targets: &[Self]) -> bool {
        targets.contains(&self)
    }
}

impl StageKeyU64 for Stage4 {
    fn key(self) -> u64 {
        todo!()
        // self.bits
    }
    // const PRUNING_MAP_TWISTS: &[Twist] = &Self::TWISTS;
    const PRUNING_MAP_TWISTS: &[Twist] = &[];
}

impl Stage for Stage4 {
    const TWISTS: TwistSet = Self::GENERATED_TWISTS;

    fn do_twist(self, twist: Twist) -> Self {
        self.generated_do_twist(twist)
    }

    fn from_state(state: SimplePuzzleSim) -> Self {
        let mut r = [RidgePos(0); 4];
        let mut ridge_index = 0;
        for pos in PieceType::Ridge.iter() {
            let (init, att) = state.get_piece(pos);
            let is_misoriented = if init[X] == 0 {
                (att * init)[X] != 0 // M -> R/L
            } else {
                X.transform_by(att) != X // R/L -> anywhere else
            };
            if is_misoriented {
                let [f1, f2] = init.facets().collect_array().unwrap();
                assert!(ridge_index < 4, "too many unsolved ridges");
                let mut ridge = RidgePos::new(f1, f2).transform_by(att);
                if ridge.facet1().axis() == X {
                    // canonicalize orientation of M ridge
                    ridge = RidgePos::new(ridge.facet2(), ridge.facet1());
                }
                r[ridge_index] = ridge;
                ridge_index += 1;
            }
        }
        assert_eq!(4, ridge_index, "not enough unsolved ridges");
        r.sort(); // put M ridges first
        assert_eq!(X, r[0].facet2().axis(), "expected 2 M ridges on R/L");
        assert_eq!(X, r[1].facet2().axis(), "expected 2 M ridges on R/L");
        assert!(r[2].is_in_m(), "expected 2 R/L ridges on M");
        assert!(r[3].is_in_m(), "expected 2 R/L ridges on M");

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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PackedTwistData(u32); // u3 + u24
impl PackedTwistData {
    pub fn facet(self) -> Facet {
        unsafe { std::hint::assert_unchecked(self.0 & 0x7FFF == self.0) };
        Facet::from_u8((self.0 >> 24) as u8)
    }

    pub fn rot_facet(self, f: Facet) -> Facet {
        Facet::from_u8(((self.0 >> f as u8) & 0x7) as u8)
    }
}

#[inline(never)]
#[unsafe(no_mangle)]
pub(crate) fn update_ridges(ridges: [RidgePos; 4], twist: Twist) -> [RidgePos; 4] {
    sort_ridge_pairs(ridges.map(|ridge| ridge.do_twist(twist)))
}

#[must_use]
fn sort_ridge_pairs(mut ridges: [RidgePos; 4]) -> [RidgePos; 4] {
    ridges[0..2].sort();
    ridges[2..4].sort();
    ridges
}

/// Returns a 6-bit integer representing a ridge position + orientation
/// (equivalently: a ridge sticker).
fn ridge_bits(init: Vec4, att: Mat4) -> RidgePos {
    let [f1, f2] = (att * init).facets().collect_array().unwrap();
    RidgePos::new(f1, f2)
}

/// Position + orientation of a ridge, represented as an ordered pair of facets.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RidgePos(u8);

impl fmt::Debug for RidgePos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("RidgePos")
            .field(&self.facet1())
            .field(&self.facet2())
            .finish()
    }
}

impl RidgePos {
    fn new(facet1: Facet, facet2: Facet) -> Self {
        Self(facet1 as u8 | ((facet2 as u8) << 3))
    }

    fn facet1(self) -> Facet {
        Facet::from_u8(self.0 & 0x7)
    }

    fn facet2(self) -> Facet {
        Facet::from_u8(self.0 >> 3)
    }

    fn is_in_m(self) -> bool {
        self.facet1().axis() != X && self.facet2().axis() != X
    }

    #[must_use]
    fn do_twist(self, twist: Twist) -> Self {
        debug_assert_eq!(0, self.0 & !0o77);
        if self.facet1() == twist.facet() {
            Self::new(self.facet1(), twist.rot() * self.facet2())
        } else if self.facet2() == twist.facet() {
            Self::new(twist.rot() * self.facet1(), self.facet2())
        } else {
            self
        }
    }
}

impl TransformByMat4 for RidgePos {
    fn transform_by(&self, m: Mat4) -> Self {
        Self::new(m * self.facet1(), m * self.facet2())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage4_ridge_pos() {
        for f1 in Facet::ALL {
            for f2 in Facet::ALL {
                let r = RidgePos::new(f1, f2);
                assert_eq!(f1, r.facet1());
                assert_eq!(f2, r.facet2());
            }
        }
    }
}
