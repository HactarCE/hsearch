use crate::linalg::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PieceType {
    /// 1 core (0-color piece)
    Core = 0,
    /// 8 centers (1-color pieces)
    Center = 1,
    /// 24 ridges (2-color pieces)
    Ridge = 2,
    /// 32 edges (3-color pieces)
    Edge = 3,
    /// 16 corners (4-color pieces)
    Corner = 4,
}

impl PieceType {
    /// Returns the number of stickers on a piece with this type.
    pub fn sticker_count(self) -> usize {
        self as _
    }

    /// Returns an iterator over all pieces with this type.
    pub fn iter(self) -> impl Iterator<Item = Vec4> {
        Vec4::region(Vec4([-1; 4]), Vec4([1; 4]))
            .filter(move |v| v.taxicab_norm() == self.sticker_count())
    }

    /// Returns an iterator over all stickers of all pieces with this type.
    pub fn all_stickers(self) -> impl Iterator<Item = Vec4> {
        super::all_stickers().filter(move |v| v.taxicab_norm() == self.sticker_count() + 1)
    }
}
