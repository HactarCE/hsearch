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
