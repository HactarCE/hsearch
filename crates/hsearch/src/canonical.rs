//! Utilities for generating canonical twist sequences.
//!
//! A twist sequence is canonical if it obeys the following rules:
//!
//! 1. A twist is never followed by another twist of the same facet.
//! 2. A twist of a negative facet is never followed by a twist of the positive
//!    facet on the same axis.

use crate::prelude::*;

/// State of a canonical twist sequence generator.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PrevTwists(Option<Facet>);

impl PrevTwists {
    /// Constructs the state for an empty canonical twist sequence.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the new state after applying a twist, or `None` if `twist` is
    /// not the next element in any canonical twist sequence.
    #[must_use]
    pub fn do_twist(self, twist: Twist) -> Option<Self> {
        self.do_twist_on_facet(twist.facet())
    }

    /// Returns the new state after applying a twist on the given facet, or
    /// `None` if such a twist cannot be the next element in any canonical twist
    /// sequence.
    pub fn do_twist_on_facet(self, facet: Facet) -> Option<Self> {
        match self.0 {
            Some(last_facet) => {
                last_facet.axis() != facet.axis()
                    || (last_facet.sign() == Sign::Pos && facet.sign() == Sign::Neg)
            }
            None => true,
        }
        .then_some(Self(Some(facet)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prev_twists_do_twist() {
        let p = PrevTwists::new();
        for f in Facet::ALL {
            assert!(p.do_twist_on_facet(f).is_some());
        }
        let after_f = p.do_twist_on_facet(F).unwrap();
        let after_b = p.do_twist_on_facet(B).unwrap();
        assert!(after_f.do_twist_on_facet(U).is_some()); // Some
        assert!(after_f.do_twist_on_facet(D).is_some()); // Some
        assert!(after_f.do_twist_on_facet(F).is_none());
        assert!(after_f.do_twist_on_facet(B).is_some()); // Some
        assert!(after_b.do_twist_on_facet(U).is_some()); // Some
        assert!(after_b.do_twist_on_facet(D).is_some()); // Some
        assert!(after_b.do_twist_on_facet(F).is_none());
        assert!(after_b.do_twist_on_facet(B).is_none());
    }
}
