use crate::prelude::*;

mod s1_pps;

pub use s1_pps::Stage1;

pub trait Stage: Copy + Default {
    /// Applies a twist and returns the new state.
    #[must_use]
    fn do_twist(self, twist_index: Twist) -> Self;

    /// Returns a state with a given scramble.
    ///
    /// # Panics
    ///
    /// Panics if a twist is unrepresentable for this stage.
    fn with_setup(twists: &[Twist]) -> Self {
        twists
            .into_iter()
            .fold(Self::default(), |state, &twist| state.do_twist(twist))
    }
}
