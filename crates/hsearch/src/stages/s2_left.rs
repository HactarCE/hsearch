use bnum::{cast::As, types::U256};
use num_traits::{One, Zero};

use super::*;

include!(concat!("../generated/stage2.rs"));

/// Stage 2: Left block (1x3x3x2)
///
/// ## Invariants
///
/// - All `I`/`O` ridges must remain oriented.
/// - The 1x3x3x2 block of `M` pieces in `M` at `[0, -1, -1, 0]..=[0, 1, 1, 1]`
///   (i.e., `~(R | L | I)`) must be setwise-preserved.
///
/// ## Move set
///
/// 80 twists are allowed:
///
/// - All `R`, `L`, and `I` twists (69 twists)
/// - `UO2`, `DO2`, `FO2`, and `BO2` (4 twists)
/// - `OR`, `OR2`, and `OL` (3 twists)
/// - `OU2`, `OF2`, `OUF`, and `ODF` (4 twists)
///
/// ## Target
///
/// - 1x3x3x2 block of `R`/`L`-oriented pieces in `L` (`[-1, -1, -1, 0]..=[-1,
///   1, 1, 1]`)
///     - 5 ridges (already oriented from stage 1)
///     - 8 oriented `I`/`O` edges
///     - 4 oriented corners
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Stage2 {
    /// For each `R`/`L`/`I` ridge location, 1 bit indicating one of the
    /// following cases:
    ///
    /// - `1` = belongs in `R`/`L`
    /// - `0` = belongs in `M` slice
    ///
    /// For each sticker of each `R`/`L`/`I` edge location, 1 bit indicating one
    /// of the following cases:
    ///
    /// - `1` = `R`/`L` sticker
    /// - `0` = other sticker
    pub re: u128, // (u16 for ridges, u84 for edges)

    /// For each sticker of each `R`/`L`/`I` corner location, 1 bit indicating
    /// one of the following cases:
    ///
    /// - `1` = `R`/`L` sticker
    /// - `0` = other sticker
    pub c: u64, // u64
}

impl Default for Stage2 {
    fn default() -> Self {
        Self::SOLVED
    }
}

impl Stage2 {
    pub fn is_target_solved(self) -> bool {
        self.re & Self::TARGET.re == Self::TARGET.re && self.c & Self::TARGET.c == Self::TARGET.c
    }
}

impl Stage for Stage2 {
    const TWISTS: TwistSet = Self::GENERATED_TWISTS;

    fn do_twist(self, twist: Twist) -> Self {
        self.generated_do_twist(twist)
    }

    fn from_state(state: SimplePuzzleSim) -> Self {
        let is_in_stage2 = |v: Vec4| v[X] != 0 || v[W] < 0;

        let r = state.pieces_to_bits(1, &[PieceType::Ridge], is_in_stage2, |init, _att| {
            (init[X] != 0) as u64
        }) as u128;

        let edge_stickers = PieceType::Edge.all_stickers().filter(|&v| is_in_stage2(v));
        let e = collect_bits::<u128>(edge_stickers.map(|v| state.is_sticker_from_axis(v, X)));

        let corner_stickers = PieceType::Corner
            .all_stickers()
            .filter(|&v| is_in_stage2(v));
        let c = collect_bits::<u64>(corner_stickers.map(|v| state.is_sticker_from_axis(v, X)));

        let re = r | (e << 16);
        Self { re, c }
    }
}

impl SubsetMaskStage for Stage2 {
    type Key = Stage2TrieKey;
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Stage2TrieKey(U256);

impl From<Stage2> for Stage2TrieKey {
    fn from(state: Stage2) -> Self {
        Self((state.re.as_::<U256>() << 64) | state.c.as_::<U256>())
    }
}

impl TrieKey for Stage2TrieKey {
    const BITS: u8 = 16 + 84 + 64;

    fn write_bits(
        self,
        mut bit_count: u8,
        bitbuffer: &mut bitbuffer::BitWriteStream<'_, bitbuffer::LittleEndian>,
    ) -> bitbuffer::Result<()> {
        let mut remaining = self.0;
        while bit_count > 0 {
            let chunk = remaining.as_::<usize>();
            let chunk_size = bit_count.min(usize::BITS as u8);
            bitbuffer.write_int(chunk, chunk_size as usize)?;
            bit_count -= chunk_size;
            remaining >>= usize::BITS;
        }
        Ok(())
    }

    fn read_bits(
        mut bit_count: u8,
        bitbuffer: &mut bitbuffer::BitReadStream<'_, bitbuffer::LittleEndian>,
    ) -> bitbuffer::Result<Self> {
        let mut ret = U256::zero();
        while bit_count > 0 {
            let chunk_size = bit_count.min(usize::BITS as u8);
            let chunk = bitbuffer.read_int::<usize>(chunk_size as usize)?;
            ret <<= chunk_size;
            ret |= chunk.as_::<U256>();
            bit_count -= chunk_size;
        }
        Ok(Self(ret))
    }

    fn lsb(self) -> bool {
        self.0.as_::<usize>() & 1 != 0
    }

    fn matches(self, entry_key: Self, _bit_count: u8) -> bool {
        self.0 & entry_key.0 == entry_key.0
    }

    fn skip(self, bit_count: u8) -> Self {
        Self(self.0 >> bit_count)
    }

    fn truncate(self, bit_count: u8) -> Self {
        Self(
            self.0
                & U256::one()
                    .unbounded_shl(bit_count as u32)
                    .wrapping_sub(U256::one()),
        )
    }

    fn branches(self) -> [bool; 2] {
        [true, self.lsb()]
    }

    fn common_lsb_prefix(self, other: Self) -> u8 {
        (self.0 ^ other.0).trailing_zeros() as u8
    }
}
