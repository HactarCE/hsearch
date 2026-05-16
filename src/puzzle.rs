//! Unoptimized puzzle implementation for generating lookup tables.

use std::{collections::HashMap, fmt, sync::LazyLock};

use crate::linalg::*;

/// Right (X+)
pub const R: Facet = Facet::pos(X);
/// Left (X-)
pub const L: Facet = Facet::neg(X);
/// Up (Y+)
pub const U: Facet = Facet::pos(Y);
/// Down (Y-)
pub const D: Facet = Facet::neg(Y);
/// Front (Z+)
pub const F: Facet = Facet::pos(Z);
/// Back (Z-)
pub const B: Facet = Facet::neg(Z);
/// Out (W+)
pub const O: Facet = Facet::pos(W);
/// In (W-)
pub const I: Facet = Facet::neg(W);

/// List of all twists on a hypercube puzzle.
pub static HYPERCUBE_TWISTS: LazyLock<Vec<Twist>> =
    LazyLock::new(|| TWISTS_WITH_NAMES.iter().map(|(_, twist)| *twist).collect());

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
///
/// There are 8 facets:
///
/// - Right (X+)
/// - Left (X-)
/// - Up (Y+)
/// - Down (Y-)
/// - Front (Z+)
/// - Back (Z-)
/// - Out (W+)
/// - In (W-)
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Facet {
    pub axis: Axis,
    pub sign: Sign,
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
        Self {
            axis,
            sign: Sign::from_i8(v[axis]),
        }
    }
}

impl Facet {
    /// Returns the positive facet on an axis.
    pub const fn pos(axis: Axis) -> Self {
        let sign = Sign::Pos;
        Self { axis, sign }
    }

    /// Returns the negative facet on an axis.
    pub const fn neg(axis: Axis) -> Self {
        let sign = Sign::Neg;
        Self { axis, sign }
    }

    /// Returns the name of the facet.
    pub const fn name(self) -> char {
        let Self { axis, sign } = self;
        match (axis, sign) {
            (X, Sign::Pos) => 'R',
            (X, Sign::Neg) => 'L',
            (Y, Sign::Pos) => 'U',
            (Y, Sign::Neg) => 'D',
            (Z, Sign::Pos) => 'F',
            (Z, Sign::Neg) => 'B',
            (W, Sign::Pos) => 'O',
            (W, Sign::Neg) => 'I',
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
        v[self.axis] == self.sign as _
    }

    /// Returns the normal vector of the facet.
    pub fn vec4(self) -> Vec4 {
        self.axis.unit() * self.sign as _
    }
}

/// Twist of an outer layer of the puzzle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Twist {
    /// Facet to twist.
    pub facet: Facet,
    /// Rotation to apply to affected pieces.
    pub rot: Mat4,
}

impl TransformByMat4 for Twist {
    fn transform_by(&self, m: Mat4) -> Self {
        Self {
            facet: self.facet.transform_by(m),
            rot: self.rot.transform_by(m),
        }
    }
}

impl fmt::Display for Twist {
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

impl Twist {
    /// Constructs a twist.
    ///
    /// # Panics
    ///
    /// Panics if `rot` does not fix `facet`.
    pub fn new(facet: Facet, rot: Mat4) -> Twist {
        assert_eq!(facet.vec4(), rot * facet.vec4(), "rot does not fix facet");
        Twist { facet, rot }
    }

    /// Returns whether the twist affects a piece given its current location.
    pub fn affects(self, location: Vec4) -> bool {
        self.facet.has_vector(location)
    }

    /// Returns a twist from its name.
    pub fn from_notation(move_str: &str) -> Option<Twist> {
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
    pub fn inv(self) -> Twist {
        Twist {
            facet: self.facet,
            rot: self.rot.inv(),
        }
    }
}

type TwistNameWithMultiplier = (Vec<Facet>, u8);

static TWISTS_WITH_NAMES: LazyLock<Vec<(TwistNameWithMultiplier, Twist)>> = LazyLock::new(|| {
    let rot_yx = Mat4::rot(Y, X);
    let rot_xz = Mat4::rot(X, Z);
    let init_twists = vec![
        ((vec![I, F], 1), Twist::new(I, rot_yx)), // 90° ridge twist
        ((vec![I, F], 2), Twist::new(I, rot_yx * rot_yx)), // 180° ridge twist
        ((vec![I, U, R], 1), Twist::new(I, rot_yx * rot_xz * rot_xz)), // 180° edge twist
        ((vec![I, U, F, R], 1), Twist::new(I, rot_yx * rot_xz)), // 120° corner twist
    ];
    let mut ret = crate::group::Group::hypercube_rotations().orbit_with(
        init_twists,
        |m, ((facets, multiplier), twist)| {
            let transformed_facets = facets.iter().map(|f| f.transform_by(m)).collect();
            ((transformed_facets, *multiplier), twist.transform_by(m))
        },
        |((_facets, _multiplier), twist)| *twist,
    );
    for ((facets, _), _) in &mut ret {
        facets[1..].sort();
    }
    ret
});

static TWIST_TO_NAME: LazyLock<HashMap<Twist, TwistNameWithMultiplier>> = LazyLock::new(|| {
    TWISTS_WITH_NAMES
        .iter()
        .map(|(name, twist)| (*twist, name.clone()))
        .collect()
});

static NAME_TO_TWIST: LazyLock<HashMap<Vec<Facet>, Twist>> = LazyLock::new(|| {
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
            assert_eq!(Twist::from_notation(&twist.to_string()), Some(*twist));
        }

        assert!(Twist::from_notation("IRU").is_some());
        assert_eq!(Twist::from_notation("IUR"), Twist::from_notation("IRU"));
        assert_eq!(Twist::from_notation("UF5"), Twist::from_notation("UF"));
        assert_eq!(Twist::from_notation("UF3"), Twist::from_notation("UF'"));
        assert_eq!(Twist::from_notation("UF2"), Twist::from_notation("UF2'"));
    }
}
