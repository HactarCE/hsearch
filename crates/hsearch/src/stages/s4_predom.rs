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
/// This target has 12 possible orientations.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Stage4 {
    /// For each ridge *sticker* location, 1 bit indicating one of the following
    /// cases:
    ///
    /// - `0` = sticker is correct for domino reduction
    /// - `1` = sticker is incorrect for domino reduction
    ///
    /// An `R`/`L` sticker is correct iff it is on `R`/`L`, and a non-`R`/`L`
    /// sticker is correct iff it is _not_ on `R`/`L`.
    ///
    /// Typically, this has exact four `1` bits.
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

impl StageKeyU128 for Stage4 {
    fn init() -> Vec<Self> {
        Self::target()
    }
    fn key(self) -> u128 {
        let r = if self.r == 0 {
            0
        } else {
            u32::from_ne_bytes(
                bit_iter::BitIter::from(self.r)
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
        let r: u64 = collect_bits(PieceType::Ridge.all_stickers().map(|sticker_vector| {
            let is_sticker_on_x = sticker_vector[X].abs() == 2;
            let is_sticker_from_x = state.is_sticker_from_axis(sticker_vector, X);
            is_sticker_on_x != is_sticker_from_x
        }));
        assert_eq!(4, r.count_ones(), "incorrect number of unsolved ridges");

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
