use super::*;

include!(concat!("../generated/stage2.rs"));

/// Stage 2: Left block (1x3x3x2)
///
/// ## Projection
///
/// This stage uses the domino projection described in the [crate] docs;
/// however, in practice only a subset of pieces needs to be tracked because
/// some remain solved under the invariants.
///
/// ## Invariants
///
/// - All `R`/`L` ridges must remain oriented.
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
/// - 1x3x3x2 block of domino-oriented pieces in `L` (`[-1, -1, -1, 0]..=[-1, 1,
///   1, 1]`)
///     - 5 ridges (already oriented from stage 1)
///     - 8 domino-oriented `R`/`L` edges
///     - 4 domino-oriented corners
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Stage2 {
    /// For each `R`/`L`/`I` ridge location, 1 bit indicating one of the
    /// following cases:
    ///
    /// - `1` = belongs in `R`/`L`
    /// - `0` = belongs in `M` slice
    ///
    /// For each `R`/`L`/`I` edge *sticker* location, 1 bit indicating one of
    /// the following cases:
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

    fn do_twist_impl(self, twist: Twist) -> OptionStage<Self> {
        self.generated_do_twist(twist).into()
    }

    fn from_state(state: SimplePuzzleSim) -> Self {
        let is_in_stage2 = |v: Vec4| !(v[X] == 0 && v[W] >= 0);

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
pub struct Stage2TrieKey([u64; 3]);

impl From<Stage2> for Stage2TrieKey {
    fn from(state: Stage2) -> Self {
        Self([state.c, state.re as u64, (state.re >> 64) as u64])
    }
}

impl Stage2TrieKey {
    fn shift_right(self, bit_count: u8) -> Self {
        let bit_count = bit_count as usize;
        if bit_count >= 192 {
            return Self([0; 3]);
        }

        let word_shift = bit_count / 64;
        let bit_shift = bit_count % 64;
        let mut ret = [0; 3];
        #[allow(clippy::needless_range_loop)] // better for clarity
        for i in 0..(3 - word_shift) {
            ret[i] = self.0[i + word_shift] >> bit_shift;
            if bit_shift > 0 && i + word_shift + 1 < 3 {
                ret[i] |= self.0[i + word_shift + 1] << (64 - bit_shift);
            }
        }
        Self(ret)
    }
}

impl TrieKey for Stage2TrieKey {
    const BITS: u8 = 16 + 84 + 64;

    fn write_bits(
        self,
        mut bit_count: u8,
        bitbuffer: &mut bitbuffer::BitWriteStream<'_, bitbuffer::LittleEndian>,
    ) -> bitbuffer::Result<()> {
        let mut word_index = 0;
        while bit_count > 0 {
            let chunk_size = bit_count.min(64);
            bitbuffer.write_int(self.0[word_index], chunk_size as usize)?;
            bit_count -= chunk_size;
            word_index += 1;
        }
        Ok(())
    }

    fn read_bits(
        mut bit_count: u8,
        bitbuffer: &mut bitbuffer::BitReadStream<'_, bitbuffer::LittleEndian>,
    ) -> bitbuffer::Result<Self> {
        let mut ret = [0; 3];
        let mut word_index = 0;
        while bit_count > 0 {
            let chunk_size = bit_count.min(64);
            ret[word_index] = bitbuffer.read_int::<u64>(chunk_size as usize)?;
            bit_count -= chunk_size;
            word_index += 1;
        }
        Ok(Self(ret))
    }

    fn lsb(self) -> bool {
        self.0[0] & 1 != 0
    }

    fn matches(self, entry_key: Self, _bit_count: u8) -> bool {
        std::iter::zip(self.0, entry_key.0).all(|(query, entry)| query & entry == entry)
    }

    fn skip(self, bit_count: u8) -> Self {
        self.shift_right(bit_count)
    }

    fn truncate(self, bit_count: u8) -> Self {
        if bit_count >= Self::BITS {
            return self;
        }
        let full_words = (bit_count / 64) as usize;
        let remaining_bit_count = bit_count % 64;
        let mut ret = [0; 3];
        ret[..full_words].copy_from_slice(&self.0[..full_words]);
        if remaining_bit_count > 0 {
            ret[full_words] = self.0[full_words] & ((1u64 << remaining_bit_count) - 1);
        }
        Self(ret)
    }

    fn branches(self) -> [bool; 2] {
        [true, self.lsb()]
    }

    fn common_lsb_prefix(self, other: Self) -> u8 {
        for (word_index, (left, right)) in std::iter::zip(self.0, other.0).enumerate() {
            let delta_mask = left ^ right;
            if delta_mask != 0 {
                return ((word_index * 64 + delta_mask.trailing_zeros() as usize) as u8)
                    .min(Self::BITS);
            }
        }
        Self::BITS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage2_trie_key_bits_round_trip() {
        let key = Stage2TrieKey([0x0123_4567_89ab_cdef, 0xfedc_ba98_7654_3210, (1 << 36) - 1]);

        for bit_count in [0, 1, 63, 64, 65, 127, 128, 163, 164] {
            let mut bytes = Vec::new();

            let mut writer = bitbuffer::BitWriteStream::new(&mut bytes, bitbuffer::LittleEndian);
            key.write_bits(bit_count, &mut writer).unwrap();

            let mut reader = bitbuffer::BitReadStream::new(bitbuffer::BitReadBuffer::new(
                &bytes,
                bitbuffer::LittleEndian,
            ));
            let decoded = Stage2TrieKey::read_bits(bit_count, &mut reader).unwrap();

            assert_eq!(decoded, key.truncate(bit_count));
        }
    }
}
