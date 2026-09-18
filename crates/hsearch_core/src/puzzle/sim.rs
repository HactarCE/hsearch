use itertools::Itertools;

use super::*;
use crate::linalg::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SimplePuzzleSim {
    /// For each current piece location: initial piece location, current
    /// attitude.
    pub pieces: [(Vec4, Mat4); 81],
}

impl Default for SimplePuzzleSim {
    fn default() -> Self {
        Self {
            pieces: Vec4::region(Vec4([-1; 4]), Vec4([1; 4]))
                .map(|v| (v, IDENT))
                .collect_array()
                .unwrap(),
        }
    }
}

impl SimplePuzzleSim {
    pub fn with_setup(twists: impl IntoIterator<Item = Twist>) -> Self {
        twists.into_iter().fold(Self::default(), Self::do_twist)
    }

    pub fn pieces_to_bits(
        &self,
        bits_per_piece: u8,
        piece_types: &[PieceType],
        filter_by_current_pos: impl Fn(Vec4) -> bool,
        map_init_pos_and_attitude: impl Fn(Vec4, Mat4) -> u64,
    ) -> u64 {
        piece_types
            .iter()
            .flat_map(|ty| {
                self.pieces
                    .into_iter()
                    .filter(|&(init, att)| {
                        init.taxicab_norm() == ty.sticker_count()
                            && filter_by_current_pos(att * init)
                    })
                    .map(|(init, att)| map_init_pos_and_attitude(init, att))
            })
            .rfold(0, |a, b| (a << bits_per_piece) | b)
    }

    /// Returns whether the sticker currently at the given position was
    /// originally on `test_axis`.
    pub fn is_sticker_from_axis(&self, sticker_vector: Vec4, test_axis: Axis) -> bool {
        let (piece_pos, ax) = sticker_vector.unwrap_sticker();
        let (_init, att) = self.get_piece(piece_pos);
        test_axis.transform_by(att) == ax
    }

    #[must_use]
    pub fn do_twist(self, twist: Twist) -> Self {
        let mut new_state = self;
        for (init, att) in self.pieces {
            let cur = att * init;
            let new_att = if twist.affects(cur) {
                twist.rot() * att
            } else {
                att
            };
            new_state.set_piece(init, new_att);
        }
        new_state
    }

    #[must_use]
    pub fn do_full_puzzle_rotation(self, rot: Mat4) -> Self {
        let mut new_state = self;
        for (init, att) in self.pieces {
            new_state.set_piece(init, rot * att);
        }
        new_state
    }

    fn set_piece(&mut self, init: Vec4, att: Mat4) {
        self.pieces[Self::index_of(att * init)] = (init, att);
    }

    pub fn get_piece(&self, pos: Vec4) -> (Vec4, Mat4) {
        self.pieces[Self::index_of(pos)]
    }

    fn index_of(pos: Vec4) -> usize {
        let Vec4([x, y, z, w]) = pos;
        ((x + 1) + (y + 1) * 3 + (z + 1) * 9 + (w + 1) * 27) as usize
    }
}
