use crate::{XyRot, prelude::*};

mod s1_ppsro;
mod s2_pio1;
mod s3_pio2;
mod s4_psio;

pub use s1_ppsro::Stage1;
pub use s2_pio1::Stage2;
pub use s3_pio2::Stage3;
pub use s4_psio::Stage4;

pub trait Stage: 'static + Send + Sync + std::fmt::Debug + Copy + Default + Eq {
    /// Applies a twist and returns the new state.
    #[must_use]
    fn do_twist(self, twist: Twist) -> Self;

    /// Applies multiple twists and returns the new state.
    #[must_use]
    fn do_twists(self, twists: impl IntoIterator<Item = Twist>) -> Self {
        twists.into_iter().fold(self, Self::do_twist)
    }

    /// Implicit rotation to apply to the puzzle after a twist.
    fn implicit_rotation_after_twist(_twist: Twist) -> XyRot {
        XyRot::IDENT
    }

    /// Returns a state with a given scramble.
    ///
    /// The default implementation applies the twists to a [`SimplePuzzleSim`]
    /// and then calls [`Self::from_state()`].
    ///
    /// # Panics
    ///
    /// Panics if a twist is unrepresentable for this stage.
    fn with_setup(twists: &[Twist]) -> Self {
        let mut state = SimplePuzzleSim::default();
        for &twist in twists {
            state = state.do_twist(twist);
        }
        Self::from_state(state)
    }

    /// Converts a state into the stage representation.
    ///
    /// # Panics
    ///
    /// Panics if the puzzle state does not satisfy the invariants of the stage.
    fn from_state(state: SimplePuzzleSim) -> Self;
}

pub trait SubsetMaskStage: Stage {
    type Key: TrieKey + From<Self>;
}

pub trait StageKeyU64: Stage {
    fn key(self) -> u64;
    const PRUNING_MAP_TWISTS: &[Twist];
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

    /// Returns whether to take each child at a branch, and then the key to use
    /// for each child.
    fn branch(self) -> ([bool; 2], Self);

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

    fn branch(self) -> ([bool; 2], Self) {
        ([true, self.0 & 1 == 0], Self(self.0 >> 1))
    }

    fn common_lsb_prefix(self, other: Self) -> u8 {
        (self.0 ^ other.0).trailing_zeros() as u8
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use pretty_assertions::assert_eq;
    use rand::{SeedableRng, seq::IndexedRandom};

    use crate::parse_twists;

    use super::*;

    #[test]
    fn test_all_stage_defaults() {
        test_stage_default::<Stage1>();
        test_stage_default::<Stage2>();
        test_stage_default::<Stage3>();
        test_stage_default::<Stage4>();
    }

    fn test_stage_default<S: Stage>() {
        assert_eq!(S::default(), S::from_state(SimplePuzzleSim::default()));
        assert_eq!(S::default(), S::with_setup(&[]));
    }

    #[test]
    fn test_all_stage_setups() {
        test_stage_setup::<Stage1>(&Twist::iter().collect_vec());
        test_stage_setup::<Stage2>(&Stage2::TWISTS);
        test_stage_setup::<Stage3>(&Stage3::TWISTS);
    }

    fn test_stage_setup<S: Stage>(allowed_twists: &[Twist]) {
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);
        let twists1 = allowed_twists
            .choose_iter(&mut rng)
            .unwrap()
            .copied()
            .take(100)
            .collect_vec();
        let twists2 = allowed_twists
            .choose_iter(&mut rng)
            .unwrap()
            .copied()
            .take(100)
            .collect_vec();
        let both = std::iter::chain(&twists1, &twists2).copied().collect_vec();
        assert_eq!(
            S::with_setup(&twists1).do_twists(twists2),
            S::with_setup(&both),
        );
    }

    #[test]
    fn test_stage4_setup_and_move_transformations() {
        // stage4 requires a separate test because of the implicit rotations
        let twists1 = parse_twists("IB ULB BLD FI RDB IB RUB OLDB BLD OF ID LDB"); // preserves stage5 invariants
        let twists2 = parse_twists("FO OR FO OUF IF2 FO OUFR FD OB FU OR OF2"); // assumes implicit rotations after each twist
        let twists3 = parse_twists("FO OR FO OFD IF2 FO OFLD FR OB FU OR OB2"); // same as above, but from global perspective

        assert_eq!(
            Stage4::with_setup(&twists1).do_twists(twists2),
            Stage4::with_setup(&std::iter::chain(twists1, twists3).collect_vec()),
        )
    }
}
