use super::*;

include!(concat!(env!("OUT_DIR"), "/stage1.rs"));

/// Stage 1: Partial `P` separation (3x3x2x1 block) + `I`/`O` ridge orientation
///
/// ## Invariants
///
/// There are no invariants to uphold.
///
/// ## Move set
///
/// All 184 twists are allowed.
///
/// ## Target
///
/// - 3x3x2x1 block of `P` pieces in `P` at `[-1, -1, -1, 0]..=[1, 1, 0, 0]`
///   (i.e., `~(F | O | I)`)
/// - I/O 2c pieces are oriented
///
/// This target has 12 possible orientations but, because we try all possible
/// orientations of the scramble, only one orientation of the target needs to be
/// checked.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Stage1 {
    /// For each edge, 1 bit indicating one of the following cases:
    ///
    /// - `0` = belongs in the P slice
    /// - `1` = belongs in I/O
    ///
    /// Thus we have the following masks for which `e_p & mask == 0`:
    ///
    /// - `1` = belongs in P slice
    /// - `0` = any
    pub e_p: u32, // u32

    /// For each ridge, 2 bits indicating one of the following cases:
    ///
    /// - `00` = belongs in P slice, any orientation
    /// - `01` = belongs in I/O, good orientation
    /// - `10` = belongs in I/O, bad orientation
    /// - `11` = unused
    ///
    /// Thus we have the following masks for which `r_op & mask == 0`:
    ///
    /// - `11` = belongs in P slice
    /// - `10` = belongs in P slice OR good orientation
    /// - `01` = belongs in P slice OR bad orientation
    /// - `00` = any
    ///
    /// Luckily, the mask transforms exactly the same as the value.
    pub r_op: u64, // u48
}

impl Default for Stage1 {
    fn default() -> Self {
        Self {
            e_p: Self::SOLVED_E_P,
            r_op: Self::SOLVED_R_OP,
        }
    }
}

impl Stage1 {
    pub const TARGET: Self = Self {
        e_p: Self::TARGET_E_P_MASK,
        r_op: Self::TARGET_R_OP_MASK,
    };

    pub fn is_solved(self) -> bool {
        self.e_p & Self::TARGET_E_P_MASK == 0 && self.r_op & Self::TARGET_R_OP_MASK == 0
    }
}

/// Returns the new orientation for a ridge.
///
/// - `r` = Rotation matrix to apply
/// - `v` = Old position
/// - `o` = Old orientation bits (just lowest 2 bits)
fn new_ridge_orientation(r: Mat4, v: Vec4, o: u8) -> u8 {
    /// Canonical axis order for determining ridge orientation.
    const RO_AXIS_ORDER: [Axis; 4] = [W, X, Y, Z];

    match o {
        0b00 | 0b11 => o, // P slice
        0b10 | 0b01 => {
            let old_axis = v.unwrap_first_nonzero_axis(RO_AXIS_ORDER);
            let new_axis = (r * v).unwrap_first_nonzero_axis(RO_AXIS_ORDER);
            if (r * old_axis.unit()).unwrap_single_axis() == new_axis {
                o
            } else {
                o ^ 0b11
            }
        }
        _ => unreachable!(),
    }
}

impl SubsetMaskStage for Stage1 {
    type Key = Stage1TrieKey;
}

impl Stage for Stage1 {
    fn from_state(state: SimplePuzzleSim) -> Self {
        Self {
            e_p: state.to_bits(
                1,
                &[PieceType::Edge],
                |_| true,
                |init, _att| (init[W] != 0) as u64,
            ) as u32,
            r_op: state.to_bits(
                2,
                &[PieceType::Ridge],
                |_| true,
                |init, att| new_ridge_orientation(att, init, (init[W] != 0) as u8) as u64,
            ),
        }
    }

    fn do_twist(self, twist: Twist) -> Self {
        self.generated_do_twist(twist)
    }
}
