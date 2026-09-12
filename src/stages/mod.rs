use crate::prelude::*;

mod s1_ppsro;
mod s2_pio1;
mod s3_pio2;

pub use s1_ppsro::Stage1;
pub use s2_pio1::Stage2;
pub use s3_pio2::Stage3;

pub trait Stage: Send + Sync + std::fmt::Debug + Copy + Default + Eq {
    /// Applies a twist and returns the new state.
    #[must_use]
    fn do_twist(self, twist: Twist) -> Self;

    /// Applies multiple twists and returns the new state.
    #[must_use]
    fn do_twists(self, twists: impl IntoIterator<Item = Twist>) -> Self {
        twists.into_iter().fold(self, Self::do_twist)
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
    /// Returns the target mask.
    ///
    /// This is often the same as the solved state (`Self::default()`), but
    /// often has fewer bits when only some pieces need to be solved.
    fn subset_trie_target() -> Self;
    fn subset_trie_key(self) -> u128;

    const SUBSET_TRIE_KEY_BITS: u32;
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
        // assert_eq!(
        //     S::with_setup(&crate::parse_twists("FR")),
        //     S::default().do_twist(crate::parse_twists("FR")[0]),
        // );
        assert_eq!(
            S::with_setup(&twists1).do_twists(twists2),
            S::with_setup(&both),
        );
    }
}
