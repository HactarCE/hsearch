use std::fmt;

use super::Twist;

/// Packed bitmask of allowed twists.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct TwistSet(pub [u8; 23]);

impl fmt::Display for TwistSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        let mut is_first = true;
        for twist in self.iter() {
            if !std::mem::take(&mut is_first) {
                write!(f, ", ")?;
            }
            write!(f, "{twist}")?;
        }
        write!(f, "]")?;
        Ok(())
    }
}

impl TwistSet {
    /// Set containing all twists.
    pub const ALL: Self = Self([u8::MAX; 23]);

    /// Constructs a twist set from a predicate.
    pub fn new(predicate: impl Fn(Twist) -> bool) -> Self {
        Self::from_iter(Twist::iter().filter(|&twist| predicate(twist)))
    }

    /// Filters the twist set according to a predicate.
    pub fn filter(&self, predicate: impl Fn(Twist) -> bool) -> Self {
        Self::from_iter(self.iter().filter(|&twist| predicate(twist)))
    }

    /// Returns an iterator over the twists in the set.
    pub fn iter(&self) -> impl Iterator<Item = Twist> {
        Twist::iter().filter(|&twist| self.contains(twist))
    }

    /// Returns a [`Vec`] containing all the twists in the set.
    pub fn to_vec(&self) -> Vec<Twist> {
        self.iter().collect()
    }

    /// Returns whether a twist is in the set.
    pub fn contains(&self, twist: Twist) -> bool {
        let (index, mask) = Self::byte_index_and_mask(twist);
        self.0[index] & mask != 0
    }

    fn byte_index_and_mask(twist: Twist) -> (usize, u8) {
        ((twist.index() as usize >> 3), 1 << (twist.index() & 0b111))
    }
}

impl FromIterator<Twist> for TwistSet {
    fn from_iter<T: IntoIterator<Item = Twist>>(iter: T) -> Self {
        let mut ret = Self::default();
        for twist in iter {
            let (index, mask) = Self::byte_index_and_mask(twist);
            ret.0[index] |= mask;
        }
        ret
    }
}
