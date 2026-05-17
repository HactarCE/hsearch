//! Unoptimized puzzle implementation for generating lookup tables.

use std::{collections::HashMap, fmt, sync::LazyLock};

use itertools::Itertools;

use crate::{Twist, linalg::*};

pub use Facet::{B, D, F, I, L, O, R, U};

/// List of all twists on a hypercube puzzle.
pub static HYPERCUBE_TWISTS: LazyLock<Vec<TwistData>> =
    LazyLock::new(|| TWISTS_WITH_NAMES.iter().map(|(_, twist)| *twist).collect());

pub static TWIST_DATA_TO_TWIST: LazyLock<HashMap<TwistData, Twist>> =
    LazyLock::new(|| std::iter::zip(HYPERCUBE_TWISTS.iter().copied(), Twist::iter()).collect());

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

    /// Returns an iterator over pieces with this type.
    pub fn iter(self) -> impl Iterator<Item = Vec4> {
        Vec4::region(Vec4([-1; 4]), Vec4([1; 4]))
            .filter(move |v| v.taxicab_norm() == self.sticker_count())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Sign {
    Pos = 1,
    Neg = -1,
}

impl Sign {
    /// Returns the sign of a nonzero number.
    ///
    /// # Panics
    ///
    /// Panics if `i == 0`.
    #[track_caller]
    pub fn from_i8(i: i8) -> Self {
        if i > 0 {
            Sign::Pos
        } else if i < 0 {
            Sign::Neg
        } else {
            panic!("cannot take sign of zero")
        }
    }
}

/// Facet of the puzzle.
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
    pub fn from_u8(i: u8) -> Self {
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

    /// Returns the axis of the facet.
    pub fn axis(self) -> Axis {
        match self {
            R | L => X,
            U | D => Y,
            F | B => Z,
            O | I => W,
        }
    }

    /// Returns the sign of the facet.
    pub fn sign(self) -> Sign {
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
        v[self.axis()] == self.sign() as _
    }

    /// Returns the normal vector of the facet.
    pub fn vec4(self) -> Vec4 {
        self.axis().unit() * self.sign() as _
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
            // reflection
            let mut ret = IDENT;
            ret[self.axis()][self.axis()] = -1;
            ret
        } else {
            if self.sign() == dst.sign() {
                self.axis().rot_to(dst.axis())
            } else {
                dst.axis().rot_to(self.axis())
            }
        }
    }
}

/// Twist of an outer layer of the puzzle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TwistData {
    /// Facet to twist.
    pub facet: Facet,
    /// Rotation to apply to affected pieces.
    pub rot: Mat4,
}

impl TransformByMat4 for TwistData {
    fn transform_by(&self, m: Mat4) -> Self {
        Self {
            facet: self.facet.transform_by(m),
            rot: self.rot.transform_by(m),
        }
    }
}

impl fmt::Display for TwistData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match TWIST_TO_NAME.get(self) {
            Some((facets, multiplier)) => {
                for facet in facets {
                    write!(f, "{facet}")?;
                }
                if *multiplier != 1 {
                    write!(f, "{multiplier}")?;
                }
                Ok(())
            }
            None => write!(f, "{self:?}"),
        }
    }
}

impl TwistData {
    /// Constructs a twist.
    ///
    /// # Panics
    ///
    /// Panics if `rot` does not fix `facet`.
    pub fn new(facet: Facet, rot: Mat4) -> TwistData {
        assert_eq!(facet.vec4(), rot * facet.vec4(), "rot does not fix facet");
        TwistData { facet, rot }
    }

    /// Returns whether the twist affects a piece given its current location.
    pub fn affects(self, location: Vec4) -> bool {
        self.facet.has_vector(location)
    }

    /// Returns a twist from its name.
    pub fn from_notation(move_str: &str) -> Option<TwistData> {
        let mut chars = move_str.chars().peekable();

        let mut facets_list = vec![];
        while let Some(&c) = chars.peek()
            && let Some(facet) = Facet::from_name(c)
        {
            chars.next();
            facets_list.push(facet);
        }

        let mut multiplier = 0_i8;
        if !chars.peek().is_some_and(char::is_ascii_digit) {
            multiplier = 1;
        }
        while let Some(&c) = chars.peek()
            && c.is_ascii_digit()
        {
            chars.next();
            multiplier = multiplier
                .checked_mul(10)?
                .checked_add(c as i8 - '0' as i8)?;
        }

        if chars.peek() == Some(&'\'') {
            chars.next();
            multiplier = multiplier.checked_neg()?;
        }

        if chars.next().is_some() {
            return None;
        }

        if facets_list.is_empty() {
            return None;
        }
        facets_list[1..].sort(); // canonicalize
        let mut unit_twist = *NAME_TO_TWIST.get(&facets_list)?;
        if multiplier != 1 {
            unit_twist.rot = unit_twist.rot.pow(multiplier);
        }
        Some(unit_twist)
    }

    /// Returns the inverse of a twist.
    pub fn inv(self) -> TwistData {
        TwistData {
            facet: self.facet,
            rot: self.rot.inv(),
        }
    }
}

type TwistNameWithMultiplier = (Vec<Facet>, u8);

static TWISTS_WITH_NAMES: LazyLock<Vec<(TwistNameWithMultiplier, TwistData)>> =
    LazyLock::new(|| {
        let rot_yx = Mat4::rot(Y, X);
        let rot_xz = Mat4::rot(X, Z);
        let init_twists = vec![
            ((vec![I, F], 1), TwistData::new(I, rot_yx)), // 90° ridge twist
            ((vec![I, F], 2), TwistData::new(I, rot_yx * rot_yx)), // 180° ridge twist
            (
                (vec![I, U, R], 1),
                TwistData::new(I, rot_yx * rot_xz * rot_xz),
            ), // 180° edge twist
            ((vec![I, U, F, R], 1), TwistData::new(I, rot_yx * rot_xz)), // 120° corner twist
        ];
        let mut all_twists_unsorted = crate::group::Group::hypercube_rotations().orbit_with(
            init_twists,
            |m, ((facets, multiplier), twist)| {
                let transformed_facets = facets.iter().map(|f| f.transform_by(m)).collect();
                ((transformed_facets, *multiplier), twist.transform_by(m))
            },
            |((_facets, _multiplier), twist)| *twist,
        );
        for ((facets, _), _) in &mut all_twists_unsorted {
            facets[1..].sort();
        }

        // Sort twists such that the lower 3 bits of a twist corresponds to its facet.
        let mut twists_by_facet = all_twists_unsorted
            .iter()
            .cloned()
            .into_group_map_by(|(_, twist_data)| twist_data.facet);
        let all_twists_interleaved = Facet::ALL
            .iter()
            .cycle()
            .map_while(|f| twists_by_facet.get_mut(f)?.pop())
            .collect_vec();
        assert_eq!(all_twists_interleaved.len(), all_twists_unsorted.len());

        all_twists_interleaved
    });

static TWIST_TO_NAME: LazyLock<HashMap<TwistData, TwistNameWithMultiplier>> = LazyLock::new(|| {
    TWISTS_WITH_NAMES
        .iter()
        .map(|(name, twist)| (*twist, name.clone()))
        .collect()
});

static NAME_TO_TWIST: LazyLock<HashMap<Vec<Facet>, TwistData>> = LazyLock::new(|| {
    TWISTS_WITH_NAMES
        .iter()
        .filter(|((_facets, multiplier), _twist)| *multiplier == 1)
        .map(|((facets, _multiplier), twist)| (facets.clone(), *twist))
        .collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_notation() {
        assert_eq!(HYPERCUBE_TWISTS.len(), 23 * 8);

        for twist in &*HYPERCUBE_TWISTS {
            assert_eq!(TwistData::from_notation(&twist.to_string()), Some(*twist));
        }

        assert!(TwistData::from_notation("IRU").is_some());
        assert_eq!(
            TwistData::from_notation("IUR"),
            TwistData::from_notation("IRU")
        );
        assert_eq!(
            TwistData::from_notation("UF5"),
            TwistData::from_notation("UF")
        );
        assert_eq!(
            TwistData::from_notation("UF3"),
            TwistData::from_notation("UF'")
        );
        assert_eq!(
            TwistData::from_notation("UF2"),
            TwistData::from_notation("UF2'")
        );
    }

    #[test]
    fn test_facet_mat4_to() {
        for f1 in Facet::ALL {
            for f2 in Facet::ALL {
                let m = f1.mat4_to(f2);
                assert_eq!(f1.transform_by(m), f2);
                if f1 == f2 {
                    assert_eq!(m, IDENT);
                } else if f1.axis() == f2.axis() {
                    assert_eq!(m.det(), -1);
                } else {
                    assert_eq!(m.det(), 1);
                }
            }
        }
    }
}
