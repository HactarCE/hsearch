use std::{fmt, ops::Deref};

use super::{TWIST_COUNT, TwistData};
use crate::linalg::*;

// pub(super) static TWIST_DATA_TO_TWIST: LazyLock<HashMap<TwistData, Twist>> =
//     LazyLock::new(|| std::iter::zip(super::TWIST_INDEX_TO_TWIST_DATA, Twist::iter()).collect());

/// Index for a twist, in the range `0..184`.
///
/// This type dereferences to its [`TwistData`].
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Twist(u8);

impl fmt::Display for Twist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &super::twist_names::TWIST_TO_NAME[self.index() as usize] {
            (facets, multiplier) => {
                for facet in facets {
                    write!(f, "{facet}")?;
                }
                if *multiplier != 1 {
                    write!(f, "{multiplier}")?;
                }
                Ok(())
            }
        }
    }
}

impl Twist {
    /// Constructs a twist.
    ///
    /// # Panics
    ///
    /// Panics if the twist is invalid.
    pub fn new(facet: Facet, rot: Mat4) -> Self {
        Self::from_data(TwistData::new(facet, rot))
    }

    /// Returns an iterator over all twists.
    pub fn iter() -> impl Iterator<Item = Twist> {
        (0..TWIST_COUNT).map(Self)
    }

    /// Returns the data associated with the twist.
    pub const fn data(self) -> TwistData {
        *self.data_ref()
    }
    const fn data_ref(&self) -> &TwistData {
        &super::twist_data::TWIST_INDEX_TO_TWIST_DATA[self.0 as usize]
    }

    /// Constructs a twist from its data.
    pub fn from_data(data: TwistData) -> Self {
        // invariants of `TwistData` guarantee this never panics
        super::twist_data::TWIST_DATA_TO_TWIST_INDEX[data.0 as usize]
    }

    /// Returns the index of the twist, which is a unique number in the range
    /// `0..`[`TWIST_COUNT`].
    pub fn index(self) -> u8 {
        self.0
    }

    /// Returns the name for a twist.
    pub fn name(self) -> String {
        self.to_string()
    }

    /// Returns a twist from its name, or `None` if the name does not correspond
    /// to a twist.
    pub fn from_name(s: &str) -> Option<Self> {
        let mut chars = s.chars().peekable();

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
        super::twist_names::NAME_TO_TWIST
            .get(&facets_list)
            .or_else(|| {
                // Try opposite names (e.g., DFLO -> DBRI')
                for f in &mut facets_list[1..] {
                    *f = f.opposite();
                }
                multiplier *= -1;
                super::twist_names::NAME_TO_TWIST.get(&facets_list)
            })?
            .pow(multiplier)
    }

    /// Returns a twist from its index.
    ///
    /// # Panics
    ///
    /// Panics if `i >= `[`TWIST_COUNT`].
    pub const fn from_index(i: u8) -> Self {
        if i >= TWIST_COUNT {
            panic!("twist index out of range");
        }
        Self(i)
    }

    /// Returns the combined result of repeating a twist, or `None` if the
    /// result is the identity.
    pub fn pow(self, multiplier: i8) -> Option<Self> {
        self.data().pow(multiplier).map(Self::from_data)
    }
}

impl Deref for Twist {
    type Target = TwistData;

    fn deref(&self) -> &Self::Target {
        self.data_ref()
    }
}

impl From<TwistData> for Twist {
    fn from(value: TwistData) -> Self {
        Self::from_data(value)
    }
}

impl From<Twist> for TwistData {
    fn from(value: Twist) -> Self {
        value.data()
    }
}

impl TransformByMat4 for Twist {
    fn transform_by(&self, m: Mat4) -> Self {
        Self::from_data(self.data().transform_by(m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twist_indices() {
        // All twists rotations must stabilize their facet.
        for twist in Twist::iter() {
            assert_eq!(twist.facet(), twist.rot() * twist.facet());
            assert_ne!(twist.rot(), crate::IDENT);
        }

        // All twists must be unique.
        assert_eq!(
            Twist::iter()
                .map(Twist::data)
                .collect::<std::collections::HashSet<TwistData>>()
                .len(),
            TWIST_COUNT as usize,
        );

        // Twists must be sorted.
        assert!(Twist::iter().is_sorted());
    }

    #[test]
    fn test_twist_data() {
        for twist in Twist::iter() {
            assert_eq!(twist, Twist::from_data(twist.data()));
        }
    }

    #[test]
    fn test_transform_twist() {
        let t = Twist::from_name("RO").unwrap();
        let expected = Twist::from_name("UO").unwrap();
        assert_eq!(t.transform_by(Mat4::rot(X, Y)), expected);
    }

    #[test]
    fn test_move_notation() {
        for twist in Twist::iter() {
            assert_eq!(Twist::from_name(&twist.to_string()), Some(twist));
        }

        assert!(Twist::from_name("IUR").is_some());
        assert!(Twist::from_name("UF5").is_some());
        assert!(Twist::from_name("UF3").is_some());
        assert!(Twist::from_name("UF2").is_some());
        assert_eq!(Twist::from_name("IUR"), Twist::from_name("IRU"));
        assert_eq!(Twist::from_name("UF5"), Twist::from_name("UF"));
        assert_eq!(Twist::from_name("UF3"), Twist::from_name("UF'"));
        assert_eq!(Twist::from_name("UF3"), Twist::from_name("UF5'"));
        assert_eq!(Twist::from_name("UF2"), Twist::from_name("UF2'"));

        assert_eq!(None, Twist::from_name("UF4"));
    }
}
