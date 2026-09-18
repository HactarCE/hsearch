use std::fmt;
use std::ops::{Index, IndexMut, Mul};

use super::*;

/// Identity matrix.
pub const IDENT: Mat4 = Mat4::from_cols([R, U, F, O]);

/// Packed 4x4 ±1 rotation/reflection matrix representation in 12 bits.
///
/// For each column:
/// - 1 bit indicating the sign of its nonzero entry.
/// - 2 bits indicating the position of its nonzero entry.
#[derive(Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Mat4(pub(crate) u16);

impl fmt::Debug for Mat4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mat4(")?;
        Axis::ALL.map(|axis| self.col(axis)).fmt(f)?;
        write!(f, ")")?;
        Ok(())
    }
}

impl fmt::Display for Mat4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        let mut ret = Vec4::ZERO;
        for axis in Axis::ALL {
            let f = self.col(axis);
            ret[f.axis()] = f.sign() as i8 * rhs[axis];
        }
        ret
    }
}

impl Mul for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Self) -> Self::Output {
        Mat4::from_cols(Axis::ALL.map(|axis| self * rhs.col(axis)))
    }
}

impl Mat4 {
    /// Constructs a matrix from columns.
    pub const fn from_cols(columns: [Facet; 4]) -> Self {
        Self(
            columns[0] as u16
                | (columns[1] as u16) << 3
                | (columns[2] as u16) << 6
                | (columns[3] as u16) << 9,
        )
    }

    /// Returns the bit offset for a column.
    const fn col_offset(column: Axis) -> u8 {
        column as u8 * 3
    }
    /// Returns the bitmask for a column.
    const fn col_mask(column: Axis) -> u16 {
        0x7 << Self::col_offset(column)
    }

    /// Returns a column of the matrix, represented as a facet that indicates
    /// the position of the nonzero entry and its sign (±1).
    pub const fn col(self, column: Axis) -> Facet {
        Facet::from_u8(((self.0 >> (column as u8 * 3)) & 0x7) as u8)
    }
    /// Sets a column of the matrix and returns a new matrix.
    #[must_use]
    pub const fn set_col(self, column: Axis, new_value: Facet) -> Self {
        let offset = Self::col_offset(column);
        let mask = Self::col_mask(column);
        Self((self.0 & !mask) | ((new_value as u16) << offset))
    }
    /// Negates a column of the matrix and returns a new matrix.
    #[must_use]
    const fn negate_col(self, column: Axis) -> Self {
        let offset = Self::col_offset(column);
        Self(self.0 ^ (1 << offset))
    }

    /// Constructs a rotation matrix from `ax1` to `ax2`.
    ///
    /// If `ax1 == ax2`, returns the identity matrix.
    pub const fn rot(ax1: Axis, ax2: Axis) -> Self {
        // positive must override when `ax1 == ax2`
        IDENT
            .set_col(ax2, Facet::new(ax1, Sign::Neg))
            .set_col(ax1, Facet::new(ax2, Sign::Pos))
    }

    /// Constructs a 180-degree rotation matrix in the plane spanned by `ax1`
    /// and `ax2`.
    ///
    /// If `ax1 == ax2`, returns the identity matrix.
    pub const fn rot180(ax1: Axis, ax2: Axis) -> Self {
        IDENT.negate_col(ax1).negate_col(ax2)
    }

    /// Constructs a reflection matrix through `axis`.
    pub const fn refl(axis: Axis) -> Self {
        IDENT.negate_col(axis)
    }

    /// Returns the inverse matrix.
    #[must_use]
    pub fn inv(self) -> Mat4 {
        let mut ret = IDENT;
        for axis in Axis::ALL {
            let f = self.col(axis);
            ret = ret.set_col(f.axis(), Facet::new(axis, f.sign()));
        }
        ret
    }

    /// Raises the matrix to a power.
    #[must_use]
    pub fn pow(self, power: i8) -> Mat4 {
        if power == 0 {
            return IDENT;
        }
        if power < 0 {
            return self.inv().pow(-power);
        }
        let init = self.pow(power / 2);
        let squared = init * init;
        if power % 2 == 0 {
            squared
        } else {
            squared * self
        }
    }

    pub(crate) const fn const_mul_facet(self, rhs: Facet) -> Facet {
        self.col(rhs.axis()).const_mul_sign(rhs.sign())
    }
}

impl Mul<Facet> for Mat4 {
    type Output = Facet;

    fn mul(self, rhs: Facet) -> Self::Output {
        self.const_mul_facet(rhs)
    }
}

impl Index<Axis> for Vec4 {
    type Output = i8;

    fn index(&self, index: Axis) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<Axis> for Vec4 {
    fn index_mut(&mut self, index: Axis) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

/// Trait for types that can be transformed by a matrix.
pub trait TransformByMat4 {
    /// Transforms `self` by the matrix `m`.
    #[must_use]
    fn transform_by(&self, m: Mat4) -> Self;
}

/// `m * self`
impl TransformByMat4 for Vec4 {
    fn transform_by(&self, m: Mat4) -> Self {
        m * *self
    }
}

/// `m * self * m.inv()`
impl TransformByMat4 for Mat4 {
    fn transform_by(&self, m: Mat4) -> Self {
        m * *self * m.inv()
    }
}

/// `(m * self.unit()).unwrap_single_axis()`
impl TransformByMat4 for Axis {
    fn transform_by(&self, m: Mat4) -> Self {
        (m * self.unit()).unwrap_single_axis()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_ops() {
        assert_eq!(-Vec4([10, 20, -30, -40]), Vec4([-10, -20, 30, 40]));
        let ones = Vec4([1, 2, 3, 4]);
        let tens = Vec4([10, 20, 30, 40]);
        assert_eq!(ones + tens, Vec4([11, 22, 33, 44]));
        assert_eq!(ones - tens, Vec4([-9, -18, -27, -36]));

        let mut a = tens;
        a += ones;
        assert_eq!(a, Vec4([11, 22, 33, 44]));
        a -= ones;
        assert_eq!(a, tens);

        assert_eq!(tens[X], 10);
        assert_eq!(tens[Y], 20);
        assert_eq!(tens[Z], 30);
        assert_eq!(tens[W], 40);

        assert_eq!(tens * -3, Vec4([-30, -60, -90, -120]));
        a *= -3;
        assert_eq!(a, Vec4([-30, -60, -90, -120]));
    }

    #[test]
    fn test_matrix_ops() {
        assert_eq!(IDENT * IDENT, IDENT);
        assert_eq!(IDENT * Vec4([10, 20, 30, 40]), Vec4([10, 20, 30, 40]));

        assert_eq!(
            Mat4::rot(X, Y) * Mat4::rot(W, Z),
            Mat4::from_cols([U, L, I, F]),
        );
        assert_eq!(
            Mat4::rot(W, Z) * Mat4::rot(X, Y),
            Mat4::from_cols([U, L, I, F]),
        );

        assert_eq!(
            Mat4::rot(X, Y) * Mat4::rot(Y, Z),
            Mat4::from_cols([U, F, R, O]),
        );

        assert_eq!(
            Mat4::rot(Y, Z) * Mat4::rot(X, Y),
            Mat4::from_cols([F, L, D, O]),
        );

        assert_eq!(
            Mat4::from_cols([U, L, I, B]) * Vec4([10, 20, 30, 40]),
            Vec4([-20, 10, -40, -30]),
        );
    }

    #[test]
    fn test_matrix_pow() {
        for m in [
            IDENT,
            Mat4::refl(W),
            Mat4::rot(Z, Y),
            Mat4::from_cols([U, O, L, F]),
        ] {
            assert_eq!(m.pow(0), IDENT);
            assert_eq!(m.pow(1), m);
            assert_eq!(m.pow(2), m * m);
            assert_eq!(m.pow(3), m * m * m);
            assert_eq!(m.pow(4), m * m * m * m);
            assert_eq!(m.pow(5), m * m * m * m * m);
            let mi = m.inv();
            assert_eq!(m.pow(-1), mi);
            assert_eq!(m.pow(-2), mi * mi);
            assert_eq!(m.pow(-3), mi * mi * mi);
        }
    }

    #[test]
    fn test_rotate_vector() {
        assert_eq!(Mat4::rot(W, Z) * Vec4([1, 2, 3, 4]), Vec4([1, 2, 4, -3]),);

        assert_eq!(
            Mat4::rot(W, Z) * Mat4::rot(X, Z) * Vec4([1, 2, 3, 4]),
            Vec4([-3, 2, 4, -1]),
        );

        let yx = Mat4::rot(Y, X);
        let v = Vec4([1, 2, 3, 4]);
        assert_eq!(yx * (yx * v), (yx * yx) * v)
    }

    #[test]
    fn test_matrix_invert() {
        let m = Mat4::rot(X, Y) * Mat4::rot(Z, Y);
        let v = Vec4([1, 2, 3, 4]);
        let m_inv = m.inv();
        assert_eq!(m * m_inv, IDENT);
        assert_eq!(IDENT, m * m_inv);
        assert_eq!(m_inv * (m * v), v);
    }

    #[test]
    fn test_transform_mat4() {
        assert_eq!(
            Mat4::rot(Z, X).transform_by(Mat4::rot(X, Y)),
            Mat4::rot(Z, Y),
        );
    }
}
