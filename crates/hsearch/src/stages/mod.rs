use crate::prelude::*;

mod s1_mid;
mod s2_left;
mod s3_count;
mod s4_predom;

pub use s1_mid::Stage1;
pub use s2_left::Stage2;
pub use s3_count::Stage3;
pub use s4_predom::Stage4;

pub trait Stage: 'static + Send + Sync + std::fmt::Debug + Copy + Default + Eq {
    /// Twist set that the stage is capable of representing.
    const TWISTS: TwistSet;

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

pub trait StageKeyU64: Stage {
    fn key(self) -> u64;
    const PRUNING_MAP_TWISTS: &[Twist];
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use pretty_assertions::assert_eq;
    use rand::SeedableRng;
    use rand::seq::IndexedRandom;

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
        test_stage_setup::<Stage1>(&[]);
        test_stage_setup::<Stage2>(&[]);
        test_stage_setup::<Stage3>(&[]);
        for setup in ["UF", "DF", "FU", "FO", "OU"] {
            test_stage_setup::<Stage4>(&parse_twists(setup));
        }
    }

    fn test_stage_setup<S: Stage>(shared_setup: &[Twist]) {
        S::with_setup(shared_setup); // don't panic

        let allowed_twists = S::TWISTS.to_vec();
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);
        let twists1 = shared_setup
            .iter()
            .chain(allowed_twists.choose_iter(&mut rng).unwrap().take(100))
            .copied()
            .collect_vec();
        println!("{}", twists_to_string(&twists1));
        let twists2 = (allowed_twists.choose_iter(&mut rng).unwrap().take(100))
            .copied()
            .collect_vec();
        let both = std::iter::chain(&twists1, &twists2).copied().collect_vec();
        assert_eq!(
            S::with_setup(&twists1).do_twists(twists2),
            S::with_setup(&both),
        );
    }
}
