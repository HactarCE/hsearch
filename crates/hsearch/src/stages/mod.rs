use crate::XyRot;
use crate::prelude::*;

mod s1_ppsro;
mod s2_pio1;
mod s3_pio2;
mod s4_psio;
mod v2_s1_sio;

pub use s1_ppsro::Stage1;
pub use s2_pio1::Stage2;
pub use s3_pio2::Stage3;
pub use s4_psio::Stage4;
pub use v2_s1_sio::V2Stage1;

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
    use crate::parse_twists;

    #[test]
    fn test_all_stage_defaults() {
        test_stage_default::<Stage1>();
        test_stage_default::<Stage2>();
        test_stage_default::<Stage3>();
        test_stage_default::<Stage4>();
        test_stage_default::<V2Stage1>();
    }

    fn test_stage_default<S: Stage>() {
        assert_eq!(S::default(), S::from_state(SimplePuzzleSim::default()));
        assert_eq!(S::default(), S::with_setup(&[]));
    }

    #[test]
    fn test_all_stage_setups() {
        test_stage_setup::<Stage1>(&Twist::ALL);
        test_stage_setup::<Stage2>(&Stage2::TWISTS);
        test_stage_setup::<Stage3>(&Stage3::TWISTS);
        test_stage_setup::<V2Stage1>(&Twist::ALL);
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
