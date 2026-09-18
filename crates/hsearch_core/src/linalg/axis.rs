use std::fmt;

use super::*;

/// Axis in 4-dimensional Euclidean space.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
    W = 3,
}

impl fmt::Display for Axis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            X => write!(f, "x"),
            Y => write!(f, "y"),
            Z => write!(f, "z"),
            W => write!(f, "w"),
        }
    }
}

impl Axis {
    /// List of all 4 axes in canonical order.
    pub const ALL: [Axis; 4] = [X, Y, Z, W];

    /// Returns a unit vector on the axis.
    pub const fn unit(self) -> Vec4 {
        Self::signed_unit(self, Sign::Pos)
    }

    /// Returns a unit vector on the axis with the given sign.
    pub const fn signed_unit(self, sign: Sign) -> Vec4 {
        let mut ret = Vec4::ZERO;
        ret.0[self as usize] = sign as i8;
        ret
    }

    /// Constructs an axis from a number in the range `0..4`.
    ///
    /// # Panics
    ///
    /// Panics if `i >= 4`.
    pub const fn from_u8(i: u8) -> Self {
        match i {
            0 => X,
            1 => Y,
            2 => Z,
            3 => W,
            _ => panic!("bad axis number"),
        }
    }

    /// Constructs a rotation matrix from `self` to `dst`.
    ///
    /// If `self == dst`, returns the identity matrix.
    pub const fn rot_to(self, dst: Axis) -> Mat4 {
        Mat4::rot(self, dst)
    }

    /// Returns a basis for the orthogonal complement of the axis; i.e., the
    /// three axes perpendicular to this one, in sorted order.
    pub fn orthogonal_basis(self) -> [Axis; 3] {
        match self {
            X => [Y, Z, W],
            Y => [X, Z, W],
            Z => [X, Y, W],
            W => [X, Y, Z],
        }
    }
}
