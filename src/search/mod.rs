use std::fmt;
use std::ops::RangeInclusive;

use itertools::Itertools;
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
    // let v2_s1_target1_prune = &*PRUNING_TABLES.v2_s1_sio_1;
    let v2_s1_target2_prune = &*PRUNING_TABLES.v2_s1_sio_2;
    let v2_s1_target3_prune = &*PRUNING_TABLES.v2_s1_sio_3;
    let v2_s1_target4_prune = &*PRUNING_TABLES.v2_s1_sio_4;
    let v2_s1_target5_prune = &*PRUNING_TABLES.v2_s1_sio_5;
    let v2_s1_target6_prune = &*PRUNING_TABLES.v2_s1_sio_6;

    let untransformed_partial = Partial::new(scramble);

    // let mut partials = itertools::iproduct!(
    //     Axis::ALL.map(|src| Mat4::rot(src, W)), // try doing P separation along a different axis
    //     [R, L, U, D, F, B].map(|f| f.mat4_to(F)), // try leaving a different facet unsolved instead of F
    // )
    // .map(|(alternative_p_sep, alternative_f_facet)| alternative_f_facet * alternative_p_sep)
    // .map(|m| untransformed_partial.transform_by(m))
    // .collect_vec();

    let mut partials = Axis::ALL
        .into_iter()
        .map(|src| Mat4::rot(src, W)) // try doing P separation along a different axis
        .map(|m| untransformed_partial.transform_by(m))
        .collect_vec();

    // println!("V2 Stage 1.1");
    // Iddfs::new::<V2Stage1>(
    //     &Twist::ALL,
    //     |s| s.is_target_solved(V2Stage1::TARGET1),
    //     |s, d| v2_s1_target1_prune.query_should_prune(s.into(), d),
    //     1..=4,
    // )
    // .iddfs_extend(&mut partials)?;
    // cleanup_and_display_solutions("stage 1.1", &mut partials, false);

    println!("V2 Stage 1.2");
    Iddfs::new::<V2Stage1>(
        &Twist::ALL,
        |s| s.is_target_solved(V2Stage1::TARGET2),
        |s, d| v2_s1_target2_prune.query_should_prune(s.into(), d),
        1..=6,
    )
    .iddfs_extend(&mut partials)?;
    cleanup_and_display_solutions("stage 1.2", &mut partials, true);

    println!("V2 Stage 1.3");
    Iddfs::new::<V2Stage1>(
        &Twist::ALL,
        |s| s.is_target_solved(V2Stage1::TARGET3),
        |s, d| v2_s1_target3_prune.query_should_prune(s.into(), d),
        1..=4,
    )
    .iddfs_extend(&mut partials)?;
    cleanup_and_display_solutions("stage 1.3", &mut partials, true);

    println!("V2 Stage 1.4");
    Iddfs::new::<V2Stage1>(
        &Twist::ALL,
        |s| s.is_target_solved(V2Stage1::TARGET4),
        |s, d| v2_s1_target4_prune.query_should_prune(s.into(), d),
        1..=4,
    )
    .iddfs_extend(&mut partials)?;
    cleanup_and_display_solutions("stage 1.4", &mut partials, true);

    println!("V2 Stage 1.5");
    Iddfs::new::<V2Stage1>(
        &Twist::ALL,
        |s| s.is_target_solved(V2Stage1::TARGET5),
        |s, d| v2_s1_target5_prune.query_should_prune(s.into(), d),
        1..=4,
    )
    .iddfs_extend(&mut partials)?;
    cleanup_and_display_solutions("stage 1.5", &mut partials, true);

    println!("V2 Stage 1.6");
    Iddfs::new::<V2Stage1>(
        &Twist::ALL,
        |s| s.is_target_solved(V2Stage1::TARGET6),
        |s, d| v2_s1_target6_prune.query_should_prune(s.into(), d),
        1..=5,
    )
    .iddfs_extend(&mut partials)?;
    cleanup_and_display_solutions("stage 1.6", &mut partials, true);

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
                        .map(|new_segment| partial.extend::<S>(&new_segment))
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
