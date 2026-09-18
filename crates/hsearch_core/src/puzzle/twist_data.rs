use std::fmt;

use super::TWIST_COUNT;
use crate::{Twist, linalg::*};

/// Packed twist representation in 15 bits.
///
/// Bits `0..12` contain a [`Mat4`].
/// Bits `12..15` contain a [`Facet`].
#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TwistData(pub(crate) u16);

impl fmt::Debug for TwistData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Twist({}, {})", self.facet(), self.rot())
    }
}

impl TwistData {
    const FACET_OFFSET: u8 = 12;
    const MATRIX_MASK: u16 = (1 << Self::FACET_OFFSET) - 1;

    /// Constructs a twist.
    ///
    /// # Panics
    ///
    /// Panics if the twist is invalid.
    pub const fn new(facet: Facet, rot: Mat4) -> Self {
        #[cfg(debug_assertions)]
        {
            if facet as u8 != rot.const_mul_facet(facet) as u8 {
                panic!("cannot construct twist: rotation does not fix facet");
            }
            if rot.0 == IDENT.0 {
                panic!("cannot construct twist: rotation is identity");
            }
        }

        Self(((facet as u16) << Self::FACET_OFFSET) | rot.0)
    }

    /// Returns the facet that is twisted.
    pub fn facet(self) -> Facet {
        Facet::from_u8((self.0 >> Self::FACET_OFFSET) as u8)
    }

    /// Returns the axis of the facet that is twisted.
    pub fn axis(self) -> Axis {
        self.facet().axis()
    }

    /// Returns the rotation applied to affected pieces.
    pub fn rot(self) -> Mat4 {
        Mat4(self.0 & Self::MATRIX_MASK)
    }

    /// Returns whether a point in space is affected by a twist.
    pub fn affects(self, v: Vec4) -> bool {
        self.facet().has_vector(v)
    }

    /// Returns the combined result of repeating a twist, or `None` if the
    /// result is the identity.
    pub fn pow(self, multiplier: i8) -> Option<Self> {
        let rot = self.rot().pow(multiplier);
        (rot != IDENT).then(|| Self::new(self.facet(), rot))
    }
}

impl TransformByMat4 for TwistData {
    fn transform_by(&self, m: Mat4) -> Self {
        Self::new(self.facet().transform_by(m), self.rot().transform_by(m))
    }
}

/// List of all twists, sorted primarily by 3D rotation matrix and secondarily
/// by facet.
pub(super) static TWIST_INDEX_TO_TWIST_DATA: [TwistData; TWIST_COUNT as usize] = {
    let mut all_twist_u16s = [0; 8 * 23];
    let mut facet_index = 0;
    let mut twist_index = 0;
    while facet_index < 8 {
        let facet = Facet::from_u8(facet_index);
        // Get the three axes perpendicular to `f.axis()`.
        let x = if facet.axis() as u8 > X as u8 { X } else { Y };
        let y = if facet.axis() as u8 > Y as u8 { Y } else { Z };
        let z = if facet.axis() as u8 > Z as u8 { Z } else { W };
        // Enumerate all non-identity 3D rotation matrices in sorted order.
        let permutations = permutations_of_3_elems([x, y, z]);
        let mut permutation_index = 0;
        while permutation_index < 6 {
            let (perm, is_odd) = permutations[permutation_index];
            let mut signs: u8 = 0;
            while signs < 8 {
                if (signs.count_ones() % 2 == 1) == is_odd {
                    let rot = IDENT
                        .set_col(x, Facet::new(perm[0], Sign::from_lsb(signs)))
                        .set_col(y, Facet::new(perm[1], Sign::from_lsb(signs >> 1)))
                        .set_col(z, Facet::new(perm[2], Sign::from_lsb(signs >> 2)));
                    if rot.0 != IDENT.0 {
                        all_twist_u16s[twist_index] = TwistData::new(facet, rot).0;
                        twist_index += 1;
                    }
                }
                signs += 1;
            }
            permutation_index += 1;
        }
        facet_index += 1;
    }

    if twist_index != all_twist_u16s.len() {
        panic!("bad total twist count");
    }

    let sorted_u16s = sort_const::const_quicksort!(all_twist_u16s);
    let mut ret = [TwistData(0); 8 * 23];
    let mut i = 0;
    while i < 8 * 23 {
        ret[i] = TwistData(sorted_u16s[i]);
        i += 1;
    }
    ret
};

pub(super) static TWIST_DATA_TO_TWIST_INDEX: [Twist; 1 << 15] = {
    let mut ret = [Twist::from_index(0); 1 << 15];
    let mut i = 0;
    while i < TWIST_COUNT {
        let data = TWIST_INDEX_TO_TWIST_DATA[i as usize];
        ret[data.0 as usize] = Twist::from_index(i);
        i += 1;
    }
    ret
};

/// Returns the 6 permutations of 3 elements, each with a boolean indicating
/// whether the permutation is odd.
const fn permutations_of_3_elems<T: Copy>([a, b, c]: [T; 3]) -> [([T; 3], bool); 6] {
    [
        ([b, c, a], false),
        ([c, b, a], true),
        ([a, c, b], true),
        ([c, a, b], false),
        ([a, b, c], false),
        ([b, a, c], true),
    ]
}
