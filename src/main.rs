//! Automated solver for 3×3×3×3 4-dimensional Rubik's cube.
//!
//! ## Puzzle description
//!
//! ### Facets
//!
//! The 3×3×3×3 puzzle exists in 4-dimensional Euclidean space. The 4 axes of
//! the space are [`X`], [`Y`], [`Z`], and [`W`].
//!
//! The puzzle has 8 [`Facet`]s, also called cells. Each assigned a letter.
//! Additionally, there are 4 slices between facets, which are also assigned
//! letters.
//!
//! | Axis  | Positive facet | Slice layer  | Negative facet |
//! | ----- | -------------- | ------------ | -------------- |
//! | [`X`] | [`R`]ight      | `M`iddle     | [`L`]eft       |
//! | [`Y`] | [`U`]p         | `E`quitorial | [`D`]own       |
//! | [`Z`] | [`F`]ront      | `S`tanding   | [`B`]ack       |
//! | [`W`] | [`O`]ut        | `P`lanetary  | [`I`]n         |
//!
//! ### Piece types
//!
//! Pieces are named based on their corresponding polytope element and on the
//! number of stickers/colors (`N`c where `N` is the number of colors).
//!
//! | Piece type | Element    | Stickers per piece | Piece count |
//! | ---------- | ---------- | ------------------ | ----------- |
//! | Core       |            | 0                  | 1           |
//! | Center     | facet/cell | 1                  | 6           |
//! | Ridge      | ridge/face | 2                  | 24          |
//! | Edge       | edge       | 3                  | 32          |
//! | Corner     | vertex     | 4                  | 16          |
//! | **Total**  |            |                    | **81**      |
//!
//! ### Twists
//!
//! Each twist is specified using 2-4 facet letters, sometimes followed by the
//! number `2`. The first letters indicate which facet the twist moves, and the
//! remaining letters indicate a perpendicular vector that is fixed by the
//! twist.
//!
//! For example, `IUR` indicates a twist of the [`I`] facet that fixes the
//! vector between [`U`] and [`R`]. In particular, it is the minimal clockwise
//! twist around the plane spanned by the normal vector of the [`I`] facet and
//! the average of the nomral vectors of the [`U`] and [`R`] facets.
//!
//! The number `2` following a turn indicates to perform the twist twice. For
//! example, `DF2` is equivalent to the sequence `DF DF`.
//!
//! ### State space
//!
//! #### Permutation
//!
//! The core and centers do not move. The remaining piece types are:
//!
//! | Piece type | Piece count |
//! | ---------- | ----------- |
//! | Ridge      | 24          |
//! | Edge       | 32          |
//! | Corner     | 16          |
//!
//! The following invariants apply:
//!
//! - The permutation parity of the corners is always even.
//! - The combined permutation parity of the ridges and edges is always even.
//!
//! This results in 24! × 32! × 16! ÷ 4 =
//! **853958999428346670146637167842740671369017657685146022707200000000000000**
//! permutation states. This is approximately 8.5396 × 10<sup>71</sup>.
//!
//! #### Orientation
//!
//! | Piece type | Piece count | Orientations | Distinguishable orientations |
//! | ---------- | ----------- | ------------ | ---------------------------- |
//! | Core       | 1           | 24           | 1                            |
//! | Center     | 6           | 8            | 1                            |
//! | Ridge      | 24          | 8            | 2                            |
//! | Edge       | 32          | 6            | 6                            |
//! | Corner     | 16          | 12           | 12                           |
//!
//! There are several invariants:
//!
//! - When all but one ridge is solved, the final unsolved ridge must be
//!   correctly oriented.
//! - When all but one corner is solved, the final unsolved corner cannot have
//!   its 3 stickers cycled (but they may have 2 pairs swapped).
//! - When all but one edge is solved, the final edge cannot have 2 of its
//!   stickers swapped (but it may have 3 of its stickers cycled).
//!
//! This results in 2<sup>23</sup> × 6<sup>31</sup> × 3 × 12<sup>15</sup> × 4 =
//! **2057209868254970923151671295243454844140433440768** orientation states.
//! This is approximately 2.05721 × 10<sup>48</sup>.
//!
//! #### Total
//!
//! Multiplying the permutation states by the orientation states yields
//! **1756772880709135843168526079081025059614484630149557651477156021733236798970168550600274887650082354207129600000000000000**
//! total puzzle states. This is approximately 1.75677 × 10<sup>120</sup>.
//!
//! Given any one of these states, the solver computes a sequence of moves that
//! takes it to the solved state.
//!
//! ## Stages
//!
//! At the time of writing, the solve stages are still being worked out.

#[macro_use]
mod lut;
mod canonical;
mod group;
mod linalg;
mod lut_gen;
mod prune;
mod puzzle;
mod search;
mod stages;
mod twist;
mod util;

pub use prelude::*;

use crate::prune::SubsetTrie;

/// Common imports.
pub mod prelude {
    pub use crate::linalg::*;
    pub use crate::puzzle::*;
    pub use crate::twist::*;
}

pub const SCRAMBLE_LEN: usize = 50;

fn main() {
    for i in 0..100 {
        let scramble = util::scramble(i);
        // println!("{}", util::twists_to_string(&scramble));
        let sol = search::solve(scramble);
        // println!("{}", util::twists_to_string(&sol));
        println!();
    }
}
