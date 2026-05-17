use crate::prelude::*;

mod s1_ppsro;

pub use s1_ppsro::Stage1;

pub trait Stage: Copy + Default + std::fmt::Debug + Send + Sync {
    /// Returns whether the stage is solved.
    fn is_solved(self) -> bool;

    /// Applies a twist and returns the new state.
    #[must_use]
    fn do_twist(self, twist: Twist) -> Self;

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

pub trait SubsetMaskStage: Stage {
    /// Returns the target mask.
    ///
    /// This is often the same as the solved state (`Self::default()`), but
    /// often has fewer bits when only some pieces need to be solved.
    fn subset_trie_target() -> Self;
    fn subset_trie_key(self) -> u128;

    const SUBSET_TRIE_KEY_BITS: u32;
}
