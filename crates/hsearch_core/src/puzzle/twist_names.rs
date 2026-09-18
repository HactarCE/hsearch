use std::{collections::HashMap, sync::LazyLock};

use super::*;
use crate::linalg::*;

type TwistNameWithMultiplier = (Vec<Facet>, u8);

static TWISTS_WITH_NAMES: LazyLock<Vec<(TwistNameWithMultiplier, Twist)>> = LazyLock::new(|| {
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
    let mut all_twists = crate::group::Group::hypercube_rotations().orbit_with(
        init_twists,
        |m, ((facets, multiplier), twist)| {
            let transformed_facets = facets.iter().map(|f| f.transform_by(m)).collect();
            ((transformed_facets, *multiplier), twist.transform_by(m))
        },
        |((_facets, _multiplier), twist)| *twist,
    );
    for ((facets, _), _) in &mut all_twists {
        facets[1..].sort();
    }

    all_twists
        .into_iter()
        .map(|(name_with_multiplier, twist_data)| (name_with_multiplier, Twist::from(twist_data)))
        .collect()
});

pub(super) static TWIST_TO_NAME: LazyLock<Vec<TwistNameWithMultiplier>> = LazyLock::new(|| {
    let mut ret = vec![TwistNameWithMultiplier::default(); TWIST_COUNT as usize];
    for (name, twist) in &*TWISTS_WITH_NAMES {
        ret[twist.index() as usize] = name.clone();
    }
    ret
});

pub(super) static NAME_TO_TWIST: LazyLock<HashMap<Vec<Facet>, Twist>> = LazyLock::new(|| {
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
    fn test_twists_round_trip() {
        for t in Twist::iter() {
            assert_eq!(vec![t], parse_twists(&t.to_string()))
        }
    }
}
