use std::ops::RangeInclusive;

use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::prelude::*;
use crate::stages::*;

mod partial;

use partial::Partial;

const SOLUTIONS_TO_DISPLAY: usize = 1;

pub fn solve(scramble: Vec<Twist>) -> Result<(), ()> {
    let s1_pps_prune = &*PRUNING_TABLES.s1_pps;

    let untransformed_partial = Partial::new(scramble);

    let mut partials = itertools::iproduct!(
        Axis::ALL.map(|src| Mat4::rot(src, W)), // try doing P separation along a different axis
        [R, L, U, D, F, B].map(|f| f.mat4_to(F)), // try leaving a different facet unsolved instead of F
    )
    .map(|(alternative_p_sep, alternative_f_facet)| alternative_f_facet * alternative_p_sep)
    .map(|m| untransformed_partial.transform_by(m))
    .collect_vec();

    println!("Stage 1");
    Iddfs::new::<Stage1>(
        &Twist::iter().collect_vec(),
        |s| s.is_solved(),
        |s, d| s1_pps_prune.query_should_prune(s.subset_trie_key(), d),
        3..=6,
    )
    .iddfs_extend(&mut partials)?;
    cleanup_and_display_solutions("stage 1", &mut partials, false);

    println!("Stage 2.1");
    Iddfs::new::<Stage2>(
        &Stage2::TWISTS,
        |s| s.is_target_solved(Stage2::TARGET1),
        |_, _| false,
        1..=4,
    )
    .iddfs_extend(&mut partials)?;
    cleanup_and_display_solutions("stage 2.1", &mut partials, false);

    println!("Stage 2.2");
    Iddfs::new::<Stage2>(
        &Stage2::TWISTS,
        |s| s.is_target_solved(Stage2::TARGET2),
        |_, _| false,
        1..=4,
    )
    .iddfs_extend(&mut partials)?;
    cleanup_and_display_solutions("stage 2.2", &mut partials, false);

    println!("Stage 2.3");
    Iddfs::new::<Stage2>(
        &Stage2::TWISTS,
        |s| s.is_target_solved(Stage2::TARGET3),
        |_, _| false,
        1..=4,
    )
    .iddfs_extend(&mut partials)?;
    cleanup_and_display_solutions("stage 2.3", &mut partials, true);

    Ok(())
}

fn cleanup_and_display_solutions(stage_name: &str, partials: &mut Vec<Partial>, verbose: bool) {
    partial::dedup_partials(partials);
    partials.sort_by_key(|p| p.len());

    println!("Found {} partial solutions to {stage_name}", partials.len());
    if verbose {
        for (i, p) in partials.iter().enumerate() {
            if i >= SOLUTIONS_TO_DISPLAY && i > 0 {
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
pub struct Iddfs<SF, PF> {
    twist_subset: Vec<Twist>,
    is_solved: SF,
    should_prune: PF,
    depth_range: RangeInclusive<u8>,
}

impl<SF, PF> Iddfs<SF, PF> {
    pub fn new<S>(
        twist_subset: &[Twist],
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
    /// `Ok(())` if successful, or `Err(())` if unsuccessful.
    pub fn iddfs_extend<S>(&self, partials: &mut Vec<Partial>) -> Result<(), ()>
    where
        S: Stage,
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
                        .map(|new_segment| partial.extend(&new_segment))
                        .collect_vec()
                })
                .collect();
            if !new_partials.is_empty() {
                *partials = new_partials;
                return Ok(());
            }
        }
        Err(())
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
        S: Stage,
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
