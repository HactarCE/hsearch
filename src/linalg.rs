//! Unoptimized linear algebra library for generating lookup tables and for
//! computations that are not performance-sensitive.

use std::fmt;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use std::ops::{Index, IndexMut};

/// Coordinate type.
pub type Coord = i8;

pub use Axis::{W, X, Y, Z};

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
        let mut ret = Vec4::ZERO;
        ret.0[self as usize] = 1;
        ret
    }

    /// Constructs an axis from a number in the range `0..4`.
    ///
    /// # Panics
    ///
    /// Panics if `i >= 4`.
    pub fn from_u8(i: u8) -> Self {
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
    pub fn rot_to(self, dst: Axis) -> Mat4 {
        Mat4::rot(self, dst)
    }
}

/// Identity matrix.
pub const IDENT: Mat4 = Mat4([Vec4::X, Vec4::Y, Vec4::Z, Vec4::W]);

/// Vector in 4-dimensional Euclidean space.
#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Vec4(pub [Coord; 4]);

impl Add for Vec4 {
    type Output = Vec4;

    fn add(self, rhs: Vec4) -> Self::Output {
        Vec4(Axis::ALL.map(|ax| self[ax] + rhs[ax]))
    }
}

impl Sub for Vec4 {
    type Output = Vec4;

    fn sub(self, rhs: Vec4) -> Self::Output {
        Vec4(Axis::ALL.map(|ax| self[ax] - rhs[ax]))
    }
}

impl AddAssign for Vec4 {
    fn add_assign(&mut self, rhs: Vec4) {
        *self = *self + rhs;
    }
}

impl SubAssign for Vec4 {
    fn sub_assign(&mut self, rhs: Vec4) {
        *self = *self - rhs;
    }
}

impl Neg for Vec4 {
    type Output = Vec4;

    fn neg(self) -> Self::Output {
        Vec4(self.0.map(|x| -x))
    }
}

impl Mul<Coord> for Vec4 {
    type Output = Vec4;

    fn mul(self, rhs: Coord) -> Self::Output {
        Vec4(self.0.map(|x| x * rhs))
    }
}

impl MulAssign<Coord> for Vec4 {
    fn mul_assign(&mut self, rhs: Coord) {
        *self = *self * rhs;
    }
}

impl Vec4 {
    /// Zero vector.
    pub const ZERO: Vec4 = Vec4([0; 4]);
    /// Unit vector along the X axis.
    pub const X: Vec4 = X.unit();
    /// Unit vector along the Y axis.
    pub const Y: Vec4 = Y.unit();
    /// Unit vector along the Z axis.
    pub const Z: Vec4 = Z.unit();
    /// Unit vector along the W axis.
    pub const W: Vec4 = W.unit();

    /// Returns the dot product of two vectors.
    pub fn dot(self, other: Vec4) -> Coord {
        std::iter::zip(self.0, other.0).map(|(a, b)| a * b).sum()
    }

    /// Returns the [taxicab](https://en.wikipedia.org/wiki/Taxicab_geometry)
    /// norm of a vector, which is the sum of the absolute values of its
    /// components.
    pub fn taxicab_norm(self) -> usize {
        self.0.map(|i| i.unsigned_abs() as usize).iter().sum()
    }

    /// Returns the single nonzero axis of this vector.
    ///
    /// # Panics
    ///
    /// Panics if the vector is zero or has multiple nonzero components.
    #[track_caller]
    pub fn unwrap_single_axis(self) -> Axis {
        let axis = Axis::from_u8(self.0.iter().position(|&x| x != 0).expect("vector is zero") as _);
        let mut v = self;
        v[axis] = 0;
        assert_eq!(v, Self::ZERO, "vector is not axis-aligned");
        axis
    }

    /// Returns an iterator over all coordinates in a region including the
    /// endpoints.
    pub fn region(min: Vec4, max: Vec4) -> impl Iterator<Item = Vec4> {
        itertools::iproduct!(
            min[W]..=max[W],
            min[Z]..=max[Z],
            min[Y]..=max[Y],
            min[X]..=max[X],
        )
        .map(|(w, z, y, x)| Vec4([x, y, z, w]))
    }

    /// Returns a list of axes that are nonzero in the vector.
    ///
    /// Axes are returned in canonical order.
    pub fn nonzero_axes(self) -> Vec<Axis> {
        Axis::ALL.into_iter().filter(|&ax| self[ax] != 0).collect()
    }

    /// Returns the first axis from `order` that is nonzero in the vector, or
    /// `None` if they are all zero.
    pub fn unwrap_first_nonzero_axis(self, order: [Axis; 4]) -> Axis {
        order
            .into_iter()
            .find(|&ax| self[ax] != 0)
            .expect("all axes are zero")
    }
}

/// Row-major matrix.
///
/// First index is row, second index is column.
#[derive(Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Mat4(pub [Vec4; 4]);

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        Vec4(self.0.map(|row| row.dot(rhs)))
    }
}

impl Mul for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Mat4) -> Self::Output {
        Mat4(Axis::ALL.map(|i| Vec4(Axis::ALL.map(|j| self.row(i).dot(rhs.col(j))))))
    }
}

impl Mat4 {
    /// Constructs a rotation matrix from `ax1` to `ax2`.
    ///
    /// If `ax1 == ax2`, returns the identity matrix.
    pub fn rot(ax1: Axis, ax2: Axis) -> Mat4 {
        let mut ret = IDENT;
        if ax1 != ax2 {
            ret[ax1][ax1] = 0;
            ret[ax2][ax2] = 0;
            ret[ax1][ax2] = -1;
            ret[ax2][ax1] = 1;
        }
        ret
    }

    /// Constructs a reflection through `axis`.
    pub fn refl(axis: Axis) -> Mat4 {
        let mut ret = IDENT;
        ret[axis][axis] = -1;
        ret
    }

    /// Constructs a matrix from its rows.
    pub fn from_rows(rows: [Vec4; 4]) -> Mat4 {
        Mat4(rows)
    }

    /// Constructs a matrix from its columns.
    pub fn from_cols(cols: [Vec4; 4]) -> Mat4 {
        Mat4(cols).t()
    }

    /// Returns the `i`th row of the matrix.
    ///
    /// # Panics
    ///
    /// Panics if `i >= 4`.
    pub fn row(self, i: Axis) -> Vec4 {
        self[i]
    }

    /// Returns the `i`th columns of the matrix.
    ///
    /// # Panics
    ///
    /// Panics if `i >= 4`.
    pub fn col(self, j: Axis) -> Vec4 {
        Vec4(self.0.map(|row| row[j]))
    }

    /// Returns the transpose matrix.
    #[must_use]
    pub fn t(self) -> Mat4 {
        Mat4(Axis::ALL.map(|ax| self.col(ax)))
    }

    /// Returns the determinant of the matrix.
    pub fn det(self) -> Coord {
        crate::util::permutations_with_parity(Axis::ALL.into_iter())
            .map(|(permutation, is_odd)| {
                let sign = if is_odd { -1 } else { 1 };
                permutation
                    .into_iter()
                    .enumerate()
                    .map(|(j, k)| self[Axis::from_u8(j as _)][k])
                    .product::<Coord>()
                    * sign
            })
            .sum()
    }

    /// Returns the inverse matrix.
    #[track_caller]
    #[must_use]
    pub fn inv(self) -> Mat4 {
        let det = self.det();
        assert_eq!(
            det.checked_abs(),
            Some(1),
            "matrix must have determinant ±1",
        );
        let det_sign = det;
        Mat4(Axis::ALL.map(|j| {
            Vec4(Axis::ALL.map(move |i| {
                let mut a = self;
                for k in Axis::ALL {
                    a[i][k] = 0;
                }
                a[i][j] = 1;
                a.det() * det_sign
            }))
        }))
    }

    /// Raises the matrix to a power.
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
}

macro_rules! impl_forward_to_debug {
    ($type:ty) => {
        impl fmt::Debug for $type {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(&self.0, f)
            }
        }

        impl fmt::Display for $type {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(&self.0, f)
            }
        }
    };
}

impl_forward_to_debug!(Vec4);
impl_forward_to_debug!(Mat4);

macro_rules! impl_index_by_axis {
    ($type:ty, $out:ty) => {
        impl Index<Axis> for $type {
            type Output = $out;

            fn index(&self, index: Axis) -> &Self::Output {
                &self.0[index as usize]
            }
        }

        impl IndexMut<Axis> for $type {
            fn index_mut(&mut self, index: Axis) -> &mut Self::Output {
                &mut self.0[index as usize]
            }
        }
    };
}

impl_index_by_axis!(Vec4, Coord);
impl_index_by_axis!(Mat4, Vec4);

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

        // modified from https://math.stackexchange.com/questions/1854288/ to fit in i8
        assert_eq!(
            Mat4([
                Vec4([5, 2, 6, 1]),
                Vec4([0, 6, 2, 0]),
                Vec4([3, 8, 1, 4]),
                Vec4([1, 8, 5, 6]),
            ]) * Mat4([
                Vec4([7, 5, 8, 0]),
                Vec4([1, 8, 2, 6]),
                Vec4([9, 4, 3, 8]),
                Vec4([5, 3, 7, -9]),
            ]),
            Mat4([
                Vec4([96, 68, 69, 51]),
                Vec4([24, 56, 18, 52]),
                Vec4([58, 95, 71, 20]),
                Vec4([90, 107, 81, 34]),
            ]),
        );

        assert_eq!(
            Mat4([
                Vec4([5, 2, 6, 1]),
                Vec4([0, 6, 2, 0]),
                Vec4([3, 8, 1, 4]),
                Vec4([1, 8, 5, 6]),
            ]) * Vec4([7, 1, 9, 5]),
            Vec4([96, 24, 58, 90]),
        );

        let m = Mat4([
            Vec4([5, 2, 6, 1]),
            Vec4([0, 6, 2, 0]),
            Vec4([3, 8, 1, 4]),
            Vec4([1, 8, 5, 6]),
        ]);
        assert_ne!(m.t(), m);
        assert_eq!(m.t().t(), m);
    }

    #[test]
    fn test_matrix_pow() {
        let m = Mat4::from_cols([
            Vec4([1, 0, 0, 0]),
            Vec4([0, -1, 2, 0]),
            Vec4([0, 2, -1, 0]),
            Vec4([0, 0, 0, 1]),
        ]);
        assert_eq!(m.pow(0), IDENT);
        assert_eq!(m.pow(1), m);
        assert_eq!(m.pow(2), m * m);
        assert_eq!(m.pow(3), m * m * m);
        assert_eq!(m.pow(4), m * m * m * m);
        assert_eq!(m.pow(5), m * m * m * m * m);
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

        assert_eq!(m.t().inv(), m.inv().t()); // just for fun
    }

    #[test]
    fn test_transform_mat4() {
        assert_eq!(
            Mat4::rot(Z, X).transform_by(Mat4::rot(X, Y)),
            Mat4::rot(Z, Y),
        );
    }
}
