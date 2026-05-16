use std::ops::Index;

use crate::puzzle::*;

/// Twist of the puzzle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Twist(u8);

impl Twist {
    /// Returns the index corresponding to the twist.
    pub fn to_index(self) -> u8 {
        self.0
    }

    /// Returns an iterator over all twists.
    pub fn iter() -> std::iter::Map<std::ops::Range<u8>, fn(u8) -> Twist> {
        (0..8 * 23).map(Self)
    }

    /// Returns the data for the twist.
    pub fn data(self) -> TwistData {
        HYPERCUBE_TWISTS[self.to_index() as usize]
    }

    /// Returns the facet of the twist.
    pub fn facet(self) -> Facet {
        Facet::from_u8(self.0 & 7)
    }
}

/// Set of twists, organized by facet.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TwistSet([Box<[Twist]>; 8]);

impl Index<Facet> for TwistSet {
    type Output = [Twist];

    fn index(&self, index: Facet) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl TwistSet {
    /// Constructs a twist set from a predicate determining whether to include
    /// the twist.
    ///
    /// If `f(twist)` returns `true`, then `twist` is included in the set. If it
    /// returns false, then `twist` is not included.
    pub fn new(mut f: impl FnMut(Twist) -> bool) -> Self {
        let mut by_facet = std::array::from_fn(|_| vec![]);
        for twist in Twist::iter() {
            if f(twist) {
                by_facet[twist.facet() as usize].push(twist);
            }
        }
        Self(by_facet.map(|vec| vec.into_boxed_slice()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twist_iter() {
        assert_eq!(Twist::iter().len(), HYPERCUBE_TWISTS.len());
    }

    #[test]
    fn test_twist_facet() {
        for t in Twist::iter() {
            assert_eq!(t.facet(), t.data().facet);
        }
    }
}
