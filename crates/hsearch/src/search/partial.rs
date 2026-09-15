use std::{collections::HashMap, fmt};

use crate::{XyRot, prelude::*, twists_to_string};

pub fn dedup_partials(partials: &mut Vec<Partial>) {
    let old_partial_count = partials.len();

    // TODO: also account for PrevTwists
    let mut state_to_index = HashMap::<SimplePuzzleSim, usize>::new();
    let mut new_partials: Vec<Partial> = vec![];
    for p in std::mem::take(partials) {
        let state = SimplePuzzleSim::with_setup(p.twists.iter().copied());
        match state_to_index.entry(state) {
            std::collections::hash_map::Entry::Occupied(e) => {
                if p.len() < new_partials[*e.get()].len() {
                    new_partials[*e.get()] = p;
                }
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(new_partials.len());
                new_partials.push(p);
            }
        }
    }

    if let Some(removed_count) = old_partial_count.checked_sub(new_partials.len())
        && removed_count > 0
    {
        println!("Removed {removed_count} duplicate solutions");
    }

    *partials = new_partials;
}

/// Scramble + partial solution.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Partial {
    /// XY rotation applied to the scramble, multiplied on the left of
    /// `scramble_rot`.
    ///
    /// This is factored out for performance reasons.
    pub scramble_xy_rot: XyRot,
    /// Rotation applied to the scramble.
    pub scramble_rot: Mat4,
    /// Scramble and solve concatenated.
    ///
    /// Applying these twists to a solved puzzle results in the latest puzzle state.
    pub twists: Vec<Twist>,
    /// Indices in `twists` that separate the scramble from the solution, and
    /// that separate the various stages/steps.
    ///
    /// This is to make printing nicer.
    pub boundaries: Vec<usize>,
}

impl fmt::Display for Partial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", twists_to_string(&self.twists))
    }
}

impl TransformByMat4 for Partial {
    fn transform_by(&self, m: Mat4) -> Self {
        Self {
            scramble_xy_rot: XyRot::IDENT,
            scramble_rot: m * self.scramble_xy_rot.mat4() * self.scramble_rot,
            twists: self.twists.iter().map(|t| t.transform_by(m)).collect(),
            boundaries: self.boundaries.clone(),
        }
    }
}

impl Partial {
    /// Constructs a new partial solution with only a scramble and no solution
    /// twists.
    pub fn new(scramble: Vec<Twist>) -> Self {
        let boundaries = vec![0, scramble.len()];
        Self {
            scramble_xy_rot: XyRot::IDENT,
            scramble_rot: IDENT,
            twists: scramble,
            boundaries,
        }
    }

    /// Returns the number of twists in the solution.
    pub fn len(&self) -> usize {
        let &scramble_end = self.boundaries.get(1).unwrap_or(&self.twists.len());
        self.twists[scramble_end..].len()
    }

    pub fn to_string_ansi(&self) -> String {
        let mut ret = String::new();
        assert_eq!(self.boundaries.first(), Some(&0));
        assert_eq!(self.boundaries.last(), Some(&self.twists.len()));
        let mut segments = self
            .boundaries
            .array_windows()
            .map(|&[start, end]| twists_to_string(&self.twists[start..end]));
        if let Some(scramble_str) = segments.next() {
            ret += "\x1B[2m"; // dim
            ret += &scramble_str;
            ret += "\x1B[0m"; // reset dim
        }
        for (i, segment_str) in segments.enumerate() {
            const BACKGROUND_COLORS: &[u8] = &[35, 34, 32, 33, 31]; // magenta, blue, green, yellow, red
            ret += &format!(" \x1B[{}m", BACKGROUND_COLORS[i % BACKGROUND_COLORS.len()]);
            ret += &segment_str;
            if segment_str.is_empty() {
                ret += ".";
            }
        }
        ret += "\x1B[0m"; // reset color
        ret += &format!(" ({} moves)", self.len());
        ret
    }

    /// Adds a segment, returning a new partial solution.
    #[must_use]
    pub fn extend<S: Stage>(&self, solution_segment: &[Twist]) -> Self {
        let mut ret = self.clone();
        for &twist in solution_segment {
            ret.twists.push(twist);
            let rot = S::implicit_rotation_after_twist(twist);
            if !rot.is_ident() {
                ret.transform_in_place_by_xy(rot);
            }
        }
        ret.boundaries.push(ret.twists.len());
        ret
    }

    /// Rotates the whole scramble and solution in place by an XY rotation.
    fn transform_in_place_by_xy(&mut self, rot: XyRot) {
        self.scramble_xy_rot = rot * self.scramble_xy_rot;
        for t in &mut self.twists {
            *t = rot.transform_twist(*t);
        }
    }
}
