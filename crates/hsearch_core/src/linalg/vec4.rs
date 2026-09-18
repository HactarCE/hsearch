use std::{
    fmt,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use itertools::Itertools;

use super::*;

/// Vector in 4-dimensional Euclidean space.
#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Vec4(pub [i8; 4]);

impl fmt::Debug for Vec4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for Vec4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

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

impl Mul<i8> for Vec4 {
    type Output = Vec4;

    fn mul(self, rhs: i8) -> Self::Output {
        Vec4(self.0.map(|x| x * rhs))
    }
}

impl MulAssign<i8> for Vec4 {
    fn mul_assign(&mut self, rhs: i8) {
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
    pub fn dot(self, other: Vec4) -> i8 {
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

    /// Returns the piece position and sticker axis of a sticker vector.
    ///
    /// A sticker vector has a single element that is ±2, and all other elements
    /// are 0 or ±1.
    #[track_caller]
    pub fn unwrap_sticker(self) -> (Vec4, Axis) {
        let axis = Axis::from_u8(self.0.iter().position_max_by_key(|x| x.abs()).unwrap() as _);
        let mut v = self;
        v[axis] /= 2;
        (v, axis)
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
    pub fn nonzero_axes(self) -> impl Iterator<Item = Axis> {
        Axis::ALL.into_iter().filter(move |&ax| self[ax] != 0)
    }

    /// Returns the facets that a vector is on.
    ///
    /// This is similar to [`Self::nonzero_axes()`], except that each axis is
    /// accompanied by a sign.
    pub fn facets(self) -> impl Iterator<Item = Facet> {
        self.nonzero_axes()
            .map(move |ax| Facet::new(ax, Sign::from_i8(self[ax])))
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
