use std::{fmt, ops::Mul};

pub use Facet::{B, D, F, I, L, O, R, U};

use crate::{Sign, linalg::*};

/// Facet of the puzzle. Also a pair `(Axis, Sign)`.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Facet {
    /// Right (X+)
    R = 0,
    /// Left (X-)
    L = 1,
    /// Up (Y+)
    U = 2,
    /// Down (Y-)
    D = 3,
    /// Front (Z+)
    F = 4,
    /// Back (Z-)
    B = 5,
    /// Out (W+)
    O = 6,
    /// In (W-)
    I = 7,
}

impl fmt::Debug for Facet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Display for Facet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl TransformByMat4 for Facet {
    fn transform_by(&self, m: Mat4) -> Self {
        let v = m * self.vec4();
        let axis = v.unwrap_single_axis();
        Self::new(axis, Sign::from_i8(v[axis]))
    }
}

impl Facet {
    /// List of all 4 axes in canonical order.
    pub const ALL: [Facet; 8] = [R, L, U, D, F, B, O, I];

    pub const fn new(axis: Axis, sign: Sign) -> Self {
        match (axis, sign) {
            (X, Sign::Pos) => R,
            (X, Sign::Neg) => L,
            (Y, Sign::Pos) => U,
            (Y, Sign::Neg) => D,
            (Z, Sign::Pos) => F,
            (Z, Sign::Neg) => B,
            (W, Sign::Pos) => O,
            (W, Sign::Neg) => I,
        }
    }

    /// Constructs a facet from a number in the range `0..8`.
    ///
    /// # Panics
    ///
    /// Panics if `i >= 8`.
    pub const fn from_u8(i: u8) -> Self {
        match i {
            0 => R,
            1 => L,
            2 => U,
            3 => D,
            4 => F,
            5 => B,
            6 => O,
            7 => I,
            _ => panic!("bad facet number"),
        }
    }

    /// Returns the positive facet on an axis.
    pub const fn pos(axis: Axis) -> Self {
        Self::new(axis, Sign::Pos)
    }

    /// Returns the negative facet on an axis.
    pub const fn neg(axis: Axis) -> Self {
        Self::new(axis, Sign::Neg)
    }

    /// Returns the opposite facet on the same axis.
    pub const fn opposite(self) -> Self {
        Self::from_u8(self as u8 ^ 1)
    }

    /// Returns the axis of the facet.
    pub const fn axis(self) -> Axis {
        match self {
            R | L => X,
            U | D => Y,
            F | B => Z,
            O | I => W,
        }
    }

    /// Returns the sign of the facet.
    pub const fn sign(self) -> Sign {
        match self {
            R | U | F | O => Sign::Pos,
            L | D | B | I => Sign::Neg,
        }
    }

    /// Returns the name of the facet.
    pub const fn name(self) -> char {
        match self {
            R => 'R',
            L => 'L',
            U => 'U',
            D => 'D',
            F => 'F',
            B => 'B',
            O => 'O',
            I => 'I',
        }
    }

    /// Returns a facet from its name, or `None` if there is no such facet.
    pub const fn from_name(c: char) -> Option<Self> {
        match c {
            'R' => Some(R),
            'L' => Some(L),
            'U' => Some(U),
            'D' => Some(D),
            'F' => Some(F),
            'B' => Some(B),
            'O' => Some(O),
            'I' => Some(I),
            _ => None,
        }
    }

    /// Returns whether the given vector is in the region of the facet.
    pub fn has_vector(self, v: Vec4) -> bool {
        v[self.axis()].signum() == self.sign() as i8
    }

    /// Returns the normal vector of the facet.
    pub fn vec4(self) -> Vec4 {
        self.axis().signed_unit(self.sign())
    }

    /// Constructs a rotation or reflection matrix from `self` to `dst`.
    ///
    /// If `self == dst`, returns the identity matrix. If `self` and `dst` are
    /// opposite, returns a reflection that takes `self` to `dst`. Otherwise,
    /// returns a 90-degree rotation matrix.
    pub fn mat4_to(self, dst: Facet) -> Mat4 {
        if self == dst {
            IDENT
        } else if self.axis() == dst.axis() {
            Mat4::refl(self.axis())
        } else {
            if self.sign() == dst.sign() {
                self.axis().rot_to(dst.axis())
            } else {
                dst.axis().rot_to(self.axis())
            }
        }
    }

    pub fn stickers(self) -> impl Iterator<Item = Vec4> {
        let mut min = Vec4([-1; 4]);
        let mut max = Vec4([1; 4]);
        min[self.axis()] = self.sign() as i8 * 2;
        max[self.axis()] = self.sign() as i8 * 2;
        Vec4::region(min, max)
    }

    pub(crate) const fn const_mul_sign(self, rhs: Sign) -> Facet {
        match rhs {
            Sign::Pos => self,
            Sign::Neg => self.opposite(),
        }
    }
}

impl Mul<Sign> for Facet {
    type Output = Facet;

    fn mul(self, rhs: Sign) -> Self::Output {
        self.const_mul_sign(rhs)
    }
}

impl Mul<Facet> for Sign {
    type Output = Facet;

    fn mul(self, rhs: Facet) -> Self::Output {
        rhs * self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_facet_mat4_to() {
        for f1 in Facet::ALL {
            for f2 in Facet::ALL {
                let m = f1.mat4_to(f2);
                assert_eq!(f1.transform_by(m), f2);
                if f1 == f2 {
                    assert_eq!(m, IDENT);
                }
            }
        }
    }
}
