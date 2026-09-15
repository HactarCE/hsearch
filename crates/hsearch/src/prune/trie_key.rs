use bnum::{cast::As, types::U256};
use num_traits::{One, Zero};

use super::*;

pub trait SubsetMaskStage: Stage {
    type Key: TrieKey + From<Self>;
}

pub trait TrieKey:
    'static + Send + Sync + std::fmt::Debug + Default + Copy + Eq + std::hash::Hash + Ord
{
    /// Number of bits.
    const BITS: u8;

    /// Writes bits to a bit buffer.
    fn write_bits(
        self,
        bit_count: u8,
        bitbuffer: &mut bitbuffer::BitWriteStream<'_, bitbuffer::LittleEndian>,
    ) -> bitbuffer::Result<()>;

    /// Reads bits from a bit buffer.
    fn read_bits(
        bit_count: u8,
        bitbuffer: &mut bitbuffer::BitReadStream<'_, bitbuffer::LittleEndian>,
    ) -> bitbuffer::Result<Self>;

    /// Returns the least significant bit as a boolean.
    fn lsb(self) -> bool;

    /// Returns whether `self` matches `entry_key` for the first `bit_count`
    /// bits.
    ///
    /// `entry_key` must already be truncated to `bit_count` bits.
    fn matches(self, entry_key: Self, bit_count: u8) -> bool;

    /// Returns `entry_key >> bit_count`.
    fn skip(self, bit_count: u8) -> Self;

    /// Returns the first `bit_count` bits of `self`.
    fn truncate(self, bit_count: u8) -> Self;

    /// Returns whether to take each child at a branch.
    fn branches(self) -> [bool; 2];

    /// Returns the length of the common prefix of `self` and `other`, starting
    /// from the least significant bit.
    fn common_lsb_prefix(self, other: Self) -> u8;
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Stage1TrieKey(u128);

impl From<Stage1> for Stage1TrieKey {
    fn from(state: Stage1) -> Self {
        Stage1TrieKey(state.e_p as u128 | ((state.r_op as u128) << 32))
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
        self.0 & entry_key.0 == 0
    }

    fn skip(self, bit_count: u8) -> Self {
        Self(self.0 >> bit_count)
    }

    fn truncate(self, bit_count: u8) -> Self {
        Self(self.0 & ((1 << bit_count) - 1))
    }

    fn branches(self) -> [bool; 2] {
        [true, !self.lsb()]
    }

    fn common_lsb_prefix(self, other: Self) -> u8 {
        (self.0 ^ other.0).trailing_zeros() as u8
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct V2Stage1TrieKey(U256);

impl From<V2Stage1> for V2Stage1TrieKey {
    fn from(state: V2Stage1) -> Self {
        Self(
            (state.r.as_::<U256>() << (64 + 96))
                | (state.e.as_::<U256>() << 64)
                | state.c.as_::<U256>(),
        )
    }
}

impl TrieKey for V2Stage1TrieKey {
    const BITS: u8 = 48 + 96 + 64;

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
