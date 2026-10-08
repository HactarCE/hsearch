use super::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Stage4WithBlock {
    pub r: u64,
    pub e: u64,
    pub c: u32,

    /// For each axis, 3 bits indicating whether the "solved block" intersects
    /// the positive, zero, and negative coordinates respectively along that
    /// axis.
    ///
    /// The region is assumed to be contiguous; i.e., the bit pattern `101` is
    /// not allowed for an axis. All other bit patterns (including `000`) are
    /// allowed.
    ///
    /// The "solved block" must never be split; a twist cannot affect only part
    /// of the solved block. If such an invalid twist is ever applied, then the
    /// `is_invalid` flag is set to `true`.
    pub solved_block: Block, // u12

    pub is_invalid: bool,
}

impl Default for Stage4WithBlock {
    fn default() -> Self {
        Self::SOLVED
    }
}

impl Stage4WithBlock {
    pub const SOLVED: Self = Self {
        r: 0,
        e: 0,
        c: 0,
        solved_block: Block::EMPTY,
        is_invalid: false,
    }
    .set_inner(Stage4::SOLVED);

    pub const INVALID: Self = Self {
        r: 0,
        e: 0,
        c: 0,
        solved_block: Block::EMPTY,
        is_invalid: true,
    };

    pub const fn inner(self) -> Stage4 {
        let Self { r, e, c, .. } = self;
        Stage4 { r, e, c }
    }
    #[must_use]
    const fn set_inner(mut self, s: Stage4) -> Self {
        self.r = s.r;
        self.e = s.e;
        self.c = s.c;
        self
    }
}

impl Stage for Stage4WithBlock {
    const TWISTS: TwistSet = Stage4::TWISTS;

    #[inline(always)]
    fn is_valid(self) -> bool {
        !self.is_invalid
    }

    fn do_twist_impl(mut self, twist: Twist) -> OptionStage<Self> {
        if !self.solved_block.is_empty() && self.solved_block.intersects_facet_layer(twist.facet())
        {
            if self.solved_block.intersects_slice_layer(twist.axis()) {
                return Self::INVALID.into();
            }
            self.solved_block = self.solved_block.transform_by(twist.rot());
        }

        self.set_inner(self.inner().do_twist_impl(twist).unwrap())
            .into()
    }

    fn from_state(state: SimplePuzzleSim) -> Self {
        let s4 @ Stage4 { r, e, c } = Stage4::from_state(state);

        Self {
            r,
            e,
            c,
            solved_block: s4
                .find_solved_223_block()
                .or_else(|| s4.find_solved_222_block())
                .unwrap_or(Block::EMPTY),
            is_invalid: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn proptest_stage4_with_block(
            init in prop::sample::select(parse_twists("UF DF UO DO FU BU FO BO OU IU OF IF")),
            twist_seq in prop::collection::vec(prop::sample::select(Stage4::TWISTS.iter().collect_vec()), 0..50)
        ) {
            test_stage4_with_block(init, twist_seq);
        }
    }

    fn test_stage4_with_block(init: Twist, twist_seq: Vec<Twist>) {
        let mut state = Stage4WithBlock::with_setup(&[init]);
        assert_eq!(12, state.solved_block.piece_count()); // 2x2x3
        for t in twist_seq {
            if let Some(s) = state.do_twist(t) {
                state = s;
            }
        }
        assert_eq!(12, state.solved_block.piece_count()); // should be preserved
    }
}
