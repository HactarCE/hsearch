use std::ops::Mul;

use crate::{
    Axis::{X, Y},
    Mat4, Twist,
};

/// Nontrivial rotation in the XY plane.
///
/// These are used by stage 4 and must be efficient to use to transform a twist
/// sequence.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct XyRot {
    /// Power of the matrix, which is always in the range `0..4`.
    pow: u8,
}

impl XyRot {
    pub const IDENT: Self = XyRot { pow: 0 };

    pub fn is_ident(self) -> bool {
        self.pow == 0
    }

    pub fn inv(self) -> Self {
        Self {
            pow: (4 - self.pow) % 4,
        }
    }

    pub fn mat4(self) -> Mat4 {
        Mat4::rot(X, Y).pow(self.pow as i8)
    }

    /// Returns the XY rotation corresponding to the matrix, or `None` for the
    /// identity.
    ///
    /// This method is relatively slow and should not be called on a hot path.
    ///
    /// # Panics
    ///
    /// Panics if the matrix does not correspond to a nontrivial rotation in the
    /// XY plane.
    pub fn from_mat4(m: Mat4) -> Self {
        (0..4)
            .map(|pow| Self { pow })
            .find(|&rot| rot.mat4() == m)
            .expect("matrix does not correspond to a rotation in the XY plane")
    }

    /// Returns the positive power `p` such that this rotation is `rot(X,
    /// Y).pow(p)`.
    ///
    /// The returned value is always less than 4.
    pub fn pow(&self) -> u8 {
        self.pow
    }

    /// Constructs an XY rotation from a power.
    pub fn from_pow(pow: u8) -> Self {
        Self { pow: pow % 4 }
    }

    /// Applies a full-puzzle rotation to a twist.
    pub fn transform_twist(self, t: Twist) -> Twist {
        if self.is_ident() {
            t
        } else {
            Twist(MUL_XY_ROT_TWIST[self.pow as usize - 1][t.to_index()])
        }
    }
}

impl Mul for XyRot {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            pow: (self.pow + rhs.pow) % 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::TransformByMat4;

    #[test]
    fn test_xy_rot_mat4_roundtrip() {
        for pow in 0..4 {
            let expected = XyRot { pow };
            let actual = XyRot::from_mat4(expected.mat4());
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_mul_xy_rot_twist() {
        for pow in 0..4 {
            for twist in Twist::iter() {
                let expected = twist.transform_by(XyRot { pow }.mat4());
                let actual = XyRot { pow }.transform_twist(twist);
                assert_eq!(expected, actual);
            }
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/xy_rot.rs"));
