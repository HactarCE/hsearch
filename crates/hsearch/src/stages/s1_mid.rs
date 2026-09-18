use super::*;

include!(concat!("../generated/stage1.rs"));

/// Stage 1: Mid block (1x3x3x2) + `I`/`O` ridge orientation
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
/// - 3x3x2x1 block of `M` pieces in `M` at `[0, -1, -1, 0]..=[0, 1, 1, 1]`
///   (i.e., `~(R | L | I)`)
/// - R/L 2c pieces are oriented
///
/// This target has 12 possible orientations but, because we try all possible
/// orientations of the scramble, only one orientation of the target needs to be
/// checked.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Stage1 {
    /// For each edge location, 1 bit indicating one of the following cases:
    ///
    /// - `1` = belongs in `M` slice
    /// - `0` = belongs in `R`/`L`
    pub e: u32, // u32

    /// For each ridge location, 2 bits indicating one of the following cases:
    ///
    /// - `11` = belongs in `M` slice, any orientation
    /// - `01` = belongs in `R`/`L`, good orientation
    /// - `10` = belongs in `R`/`L`, bad orientation
    /// - `00` = not tracked (only while building pruning table)
    ///
    /// Thus we have the following masks for which `r_op & mask == mask`:
    ///
    /// - `11` = belongs in `M` slice
    /// - `01` = belongs in `R`/`L` OR good orientation
    /// - `10` = belongs in `R`/`L` OR bad orientation
    /// - `00` = any
    ///
    /// The mask transforms exactly the same as the value.
    pub r: u64, // u48
}

impl Default for Stage1 {
    fn default() -> Self {
        Self::SOLVED
    }
}

impl Stage1 {
    pub fn is_target_solved(self) -> bool {
        self.e & Self::TARGET.e == Self::TARGET.e && self.r & Self::TARGET.r == Self::TARGET.r
    }
}

impl Stage for Stage1 {
    const TWISTS: TwistSet = Self::GENERATED_TWISTS;

    fn do_twist(self, twist: Twist) -> Self {
        self.generated_do_twist(twist)
    }

    fn from_state(state: SimplePuzzleSim) -> Self {
        Self {
            e: state.pieces_to_bits(
                1,
                &[PieceType::Edge],
                |_| true,
                |init, _att| (init[X] == 0) as u64,
            ) as u32,
            r: state.pieces_to_bits(
                2,
                &[PieceType::Ridge],
                |_| true,
                |init, att| {
                    let init_orientation = if init[X] == 0 { 0b11 } else { 0b01 };
                    hsearch_core::stage_utils::xyz_ro(att, init, init_orientation) as u64
                },
            ),
        }
    }
}

impl SubsetMaskStage for Stage1 {
    type Key = Stage1TrieKey;
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Stage1TrieKey(u128);

impl From<Stage1> for Stage1TrieKey {
    fn from(state: Stage1) -> Self {
        Stage1TrieKey(state.e as u128 | ((state.r as u128) << 32))
    }
}

impl TrieKey for Stage1TrieKey {
    const BITS: u8 = 32 + 48;

    fn write_bits(
        self,
        bit_count: u8,
        bitbuffer: &mut bitbuffer::BitWriteStream<'_, bitbuffer::LittleEndian>,
    ) -> bitbuffer::Result<()> {
        bitbuffer.write_int(self.0, bit_count as usize)
    }

    fn read_bits(
        bit_count: u8,
        bitbuffer: &mut bitbuffer::BitReadStream<'_, bitbuffer::LittleEndian>,
    ) -> bitbuffer::Result<Self> {
        bitbuffer.read_int(bit_count as usize).map(Self)
    }

    fn lsb(self) -> bool {
        self.0 & 1 != 0
    }

    fn matches(self, entry_key: Self, _bit_count: u8) -> bool {
        self.0 & entry_key.0 == entry_key.0
    }

    fn skip(self, bit_count: u8) -> Self {
        Self(self.0 >> bit_count)
    }

    fn truncate(self, bit_count: u8) -> Self {
        Self(self.0 & ((1 << bit_count) - 1))
    }

    fn branches(self) -> [bool; 2] {
        [true, self.lsb()]
    }

    fn common_lsb_prefix(self, other: Self) -> u8 {
        (self.0 ^ other.0).trailing_zeros() as u8
    }
}
