use std::fmt;
use std::ops::RangeInclusive;

use itertools::Itertools;
use rayon::iter::IntoParallelIterator;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::prelude::*;
use crate::stages::*;

mod partial;

use partial::Partial;

const SOLUTIONS_TO_DISPLAY: usize = 1;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct NoSolution;
impl fmt::Display for NoSolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "no solution")
    }
}

pub fn solve(scramble: Vec<Twist>) -> Result<(), NoSolution> {
    let s1_prune = &*PRUNING_TABLES.s1_mid;
    let s2_prune = &*PRUNING_TABLES.s2_left;

    let untransformed_partial = Partial::new(scramble);

    let mut partials = itertools::iproduct!(
        // replace M slice with any other slice
        Axis::ALL.map(|src| Mat4::rot(src, X)),
        // replace I facet with any other facet around try leaving a different facet unsolved instead of F
        [U, D, F, B, O, I].map(|f| f.mat4_to(I)),
        // TODO: do "alternative I facet" as postprocessing, with multiple targets
    )
    .map(|(alternative_p_sep, alternative_f_facet)| alternative_f_facet * alternative_p_sep)
    .map(|m| untransformed_partial.transform_by(m))
    .collect_vec();

    println!("Stage 1");
    Iddfs::new::<Stage1>(
        Stage1::TWISTS,
        |s| s.is_target_solved(),
        |s, d| s1_prune.query_should_prune(s.into(), d),
        3..=6,
    )
    .iddfs_extend(&mut partials)?;

    // Optionally swap R/L (TODO: do this as postprocessing, with multiple targets)
    partials = partials
        .into_par_iter()
        .flat_map(|partial| [partial.transform_by(Mat4::rot180(X, Y)), partial])
        .collect();

    cleanup_and_display_solutions("stage 1", &mut partials, true);

    println!("Stage 2");
    Iddfs::new::<Stage2>(
        Stage2::TWISTS,
        |s| s.is_target_solved(),
        |s, d| s2_prune.query_should_prune(s.into(), d),
        1..=6,
    )
    .iddfs_extend(&mut partials)?;
    cleanup_and_display_solutions("stage 2", &mut partials, true);

    println!("Stage 3");
    Iddfs::new::<Stage3>(
        Stage3::TWISTS,
        |s| s.is_target_solved(),
        |_, _| false,
        1..=4,
    )
    .iddfs_extend(&mut partials)?;
    cleanup_and_display_solutions("stage 3", &mut partials, true);

    println!("Stage 4");
    let target = Stage4::target();
    Iddfs::new::<Stage4>(
        Stage3::TWISTS,
        |s| s.is_target_solved(&target),
        |_, _| false,
        1..=6,
    )
    .iddfs_extend(&mut partials)?;
    cleanup_and_display_solutions("stage 4", &mut partials, true);

    // println!("Stage 2.2");
    // Iddfs::new::<Stage2>(
    //     &Stage2::TWISTS,
    //     |s| s.is_target_solved(Stage2::TARGET2),
    //     |_, _| false,
    //     1..=4,
    // )
    // .iddfs_extend(&mut partials)?;
    // cleanup_and_display_solutions("stage 2.2", &mut partials, false);

    // println!("Stage 2.3");
    // Iddfs::new::<Stage2>(
    //     &Stage2::TWISTS,
    //     |s| s.is_target_solved(Stage2::TARGET3),
    //     |_, _| false,
    //     1..=4,
    // )
    // .iddfs_extend(&mut partials)?;
    // cleanup_and_display_solutions("stage 2.3", &mut partials, false);

    // // Normalize so that the block is on `I`
    // partials.par_iter_mut().for_each(|partial| {
    //     if Stage2::with_setup(&partial.twists)
    //         .which_target3()
    //         .expect("bad solution")
    //         == Sign::Neg
    //     {
    //         *partial = partial.transform_by(Mat4::refl(W));
    //     }
    // });

    // println!("Stage 3.1");
    // Iddfs::new::<Stage3>(
    //     &Stage3::TWISTS,
    //     |s| s.is_target_solved(Stage3::TARGET1),
    //     |_, _| false,
    //     1..=4,
    // )
    // .iddfs_extend(&mut partials)?;
    // cleanup_and_display_solutions("stage 3.1", &mut partials, false);

    // println!("Stage 3.2");
    // Iddfs::new::<Stage3>(
    //     &Stage3::TWISTS,
    //     |s| s.is_target_solved(Stage3::TARGET2),
    //     |_, _| false,
    //     1..=4,
    // )
    // .iddfs_extend(&mut partials)?;

    // // Normalize so that unsolved `I`/`O` region is on `UO`.
    // partials.par_iter_mut().for_each(|partial| {
    //     let secondary_facet = Stage3::with_setup(&partial.twists)
    //         .which_target2()
    //         .expect("bad solution");
    //     if secondary_facet != Facet::U {
    //         *partial = partial.transform_by(secondary_facet.mat4_to(U));
    //     }
    // });

    // cleanup_and_display_solutions("stage 3.2", &mut partials, true);

    // println!("Stage 4");
    // Iddfs::new::<Stage4>(
    //     &Stage4::TWISTS,
    //     |s| s.is_target_solved(Stage4::SOLVED),
    //     |s, d| s4_prune.query_should_prune(s.key(), d),
    //     1..=13,
    // )
    // .iddfs_extend(&mut partials)?;
    // cleanup_and_display_solutions("stage 4", &mut partials, true);

    Ok(())
}

fn cleanup_and_display_solutions(stage_name: &str, partials: &mut Vec<Partial>, verbose: bool) {
    partial::dedup_partials(partials);
    partials.sort_by_key(|p| p.len());

    println!("Found {} partial solutions to {stage_name}", partials.len());
    if verbose {
        for (i, p) in partials.iter().enumerate() {
            if i >= SOLUTIONS_TO_DISPLAY && SOLUTIONS_TO_DISPLAY > 0 {
                let hidden_count = partials.len() - SOLUTIONS_TO_DISPLAY;
                println!("... {hidden_count} solutions not shown");
                break;
            }
            println!("{}. {}", i + 1, p.to_string_ansi());
        }
        println!();
    }
}

/// Iterative-deepening depth-first search parameters.
///
/// - `SF` = solved function ("Is this state solved?")
/// - `PF` = prune function ("Should this state be pruned?")
pub struct Iddfs<SF, PF> {
    twist_subset: Vec<Twist>,
    is_solved: SF,
    should_prune: PF,
    depth_range: RangeInclusive<u8>,
}

impl<SF, PF> Iddfs<SF, PF> {
    pub fn new<S>(
        twist_subset: TwistSet,
        is_solved: SF,
        should_prune: PF,
        depth_range: RangeInclusive<u8>,
    ) -> Self
    where
        S: Stage,
        SF: Sync + Fn(S) -> bool,
        PF: Sync + Fn(S, u8) -> bool,
    {
        Self {
            twist_subset: twist_subset.to_vec(),
            is_solved,
            should_prune,
            depth_range,
        }
    }

    /// Extends each partial using the minimum search depth necessary. Returns
    /// `Ok` if successful, or `Err` if unsuccessful.
    pub fn iddfs_extend<S: Stage>(&self, partials: &mut Vec<Partial>) -> Result<(), NoSolution>
    where
        SF: Sync + Fn(S) -> bool,
        PF: Sync + Fn(S, u8) -> bool,
    {
        for depth in self.depth_range.clone() {
            // println!("  Searching at depth {depth} ...");
            let new_partials: Vec<Partial> = partials
                .par_iter()
                .flat_map(|partial| {
                    let init = S::with_setup(&partial.twists);
                    let mut solutions = vec![];
                    self.dfs(init, PrevTwists::new(), depth, &mut vec![], &mut solutions);
                    solutions
                        .into_iter()
                        .map(|new_segment| partial.push_segment::<S>(&new_segment))
                        .collect_vec()
                })
                .collect();
            if new_partials.len() >= 100
                || depth + 1 >= *self.depth_range.end() && !new_partials.is_empty()
            {
                *partials = new_partials;
                return Ok(());
            }
        }
        Err(NoSolution)
    }

    /// Runs a depth-first search and records all solutions in `solutions`.
    ///
    /// - `solution_buffer` is the current solution segment so far
    /// - `solutions` is a collection of all complete solution segments
    fn dfs<S: Stage>(
        &self,
        state: S,
        prev_twists: PrevTwists,
        remaining_depth: u8,
        solution_buffer: &mut Vec<Twist>,
        solutions: &mut Vec<Vec<Twist>>,
    ) where
        SF: Fn(S) -> bool,
        PF: Fn(S, u8) -> bool,
    {
        if (self.is_solved)(state) {
            solutions.push(solution_buffer.clone());
            return;
        }
        if remaining_depth == 0 || (self.should_prune)(state, remaining_depth) {
            return;
        }

        for &twist in &self.twist_subset {
            let Some(new_prev_twists) = prev_twists.do_twist(twist) else {
                continue;
            };
            solution_buffer.push(twist);
            self.dfs(
                state.do_twist(twist),
                new_prev_twists,
                remaining_depth - 1,
                solution_buffer,
                solutions,
            );
            solution_buffer.pop();
        }
    }
}
