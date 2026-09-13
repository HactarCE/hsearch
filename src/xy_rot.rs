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

    use itertools::Itertools;

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

    #[test]
    fn lutgen_mul_xy_rot_twist() {
        println!(
            "const MUL_XY_ROT_TWIST: [[u8; {}]; 3] = [",
            Twist::iter().len()
        );
        for pow in [1, 2, 3] {
            let m = XyRot { pow }.mat4();
            println!(
                "    {:?},",
                Twist::iter().map(|t| t.transform_by(m).0).collect_vec(),
            );
        }
        println!("];");
    }
}

const MUL_XY_ROT_TWIST: [[u8; 184]; 3] = [
    [
        26, 3, 41, 56, 156, 29, 22, 23, 138, 51, 33, 16, 44, 133, 6, 151, 82, 99, 105, 32, 180,
        141, 86, 63, 2, 11, 129, 24, 28, 125, 102, 7, 10, 19, 1, 112, 4, 5, 110, 159, 50, 27, 121,
        72, 92, 157, 118, 127, 18, 35, 9, 40, 12, 149, 134, 15, 34, 43, 65, 80, 164, 69, 142, 31,
        42, 59, 169, 120, 108, 61, 70, 39, 98, 67, 17, 88, 20, 165, 150, 87, 58, 75, 25, 136, 36,
        85, 14, 47, 122, 83, 49, 8, 52, 181, 30, 143, 66, 91, 57, 104, 116, 173, 158, 103, 74, 107,
        73, 48, 68, 109, 46, 55, 90, 115, 145, 144, 60, 117, 38, 71, 106, 123, 81, 64, 172, 37,
        166, 79, 114, 131, 89, 0, 132, 21, 54, 167, 130, 139, 97, 152, 76, 13, 78, 95, 146, 147,
        113, 128, 148, 45, 62, 111, 154, 155, 137, 160, 84, 53, 94, 119, 162, 163, 153, 168, 100,
        77, 126, 135, 170, 171, 161, 96, 124, 93, 174, 175, 178, 179, 177, 176, 140, 101, 182, 183,
    ],
    [
        129, 56, 27, 34, 84, 125, 86, 63, 97, 40, 19, 82, 92, 21, 22, 111, 25, 104, 107, 10, 140,
        13, 14, 31, 41, 16, 131, 2, 28, 37, 158, 23, 33, 32, 3, 90, 156, 29, 46, 119, 9, 24, 123,
        98, 52, 53, 38, 79, 105, 112, 51, 50, 44, 45, 54, 151, 1, 72, 59, 58, 100, 61, 78, 7, 121,
        80, 171, 106, 68, 69, 70, 159, 57, 120, 99, 122, 180, 77, 62, 47, 65, 88, 11, 130, 4, 85,
        6, 127, 81, 136, 35, 138, 12, 101, 102, 95, 169, 8, 43, 74, 60, 93, 94, 103, 17, 48, 67,
        18, 108, 109, 118, 15, 49, 144, 147, 146, 164, 117, 110, 39, 73, 64, 75, 42, 124, 5, 126,
        87, 145, 0, 83, 26, 132, 141, 134, 135, 89, 152, 91, 154, 20, 133, 150, 143, 113, 128, 115,
        114, 148, 157, 142, 55, 137, 160, 139, 162, 36, 149, 30, 71, 153, 168, 155, 170, 116, 165,
        166, 167, 161, 96, 163, 66, 172, 181, 174, 175, 177, 176, 179, 178, 76, 173, 182, 183,
    ],
    [
        131, 34, 24, 1, 36, 37, 14, 31, 91, 50, 32, 25, 52, 141, 86, 55, 11, 74, 48, 33, 76, 133,
        6, 7, 27, 82, 0, 41, 28, 5, 94, 63, 19, 10, 56, 49, 84, 125, 118, 71, 51, 2, 64, 57, 12,
        149, 110, 87, 107, 90, 40, 9, 92, 157, 134, 111, 3, 98, 80, 65, 116, 69, 150, 23, 123, 58,
        96, 73, 108, 61, 70, 119, 43, 106, 104, 81, 140, 165, 142, 127, 59, 122, 16, 89, 156, 85,
        22, 79, 75, 130, 112, 97, 44, 173, 158, 143, 171, 138, 72, 17, 164, 181, 30, 103, 99, 18,
        120, 105, 68, 109, 38, 151, 35, 146, 128, 113, 100, 117, 46, 159, 67, 42, 88, 121, 172, 29,
        166, 47, 147, 26, 136, 129, 132, 13, 54, 167, 83, 154, 8, 137, 180, 21, 62, 95, 115, 114,
        144, 145, 148, 53, 78, 15, 139, 162, 152, 153, 4, 45, 102, 39, 155, 170, 160, 161, 60, 77,
        126, 135, 163, 66, 168, 169, 124, 101, 174, 175, 179, 178, 176, 177, 20, 93, 182, 183,
    ],
];
