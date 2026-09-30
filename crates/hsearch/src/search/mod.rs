use std::fmt;
use std::marker::PhantomData;
use std::ops::RangeInclusive;

use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

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
    let s4_prune = &*PRUNING_TABLES.s4_predom;

    let untransformed_partial = Partial::new(scramble);

    let mut partials = itertools::iproduct!(
        // replace M slice with any other slice
        Axis::ALL.map(|src| Mat4::rot(src, X)),
        // replace I facet with any other facet
        [U, D, F, B, O, I].map(|f| f.mat4_to(I)),
    )
    .map(|(alternative_p_sep, alternative_f_facet)| alternative_f_facet * alternative_p_sep)
    .map(|m| untransformed_partial.transform_by(m))
    .collect_vec();

    println!("Stage 1.1: Mid (3x2x2 block)");
    Iddfs::<Stage1, _>::new(Stage1::TWISTS, |s| s.is_target1_solved(), 1..=2)
        .iddfs_extend(&mut partials)?;
    // TODO: restrict twists for next step

    println!("Stage 1.2: Mid (3x3x2 block)");
    Iddfs::<Stage1, _>::new(Stage1::TWISTS, |s| s.is_target2_solved(), 1..=6)
        .with_prune(|s, d| s1_prune.query_should_prune(s.into(), d))
        .iddfs_extend(&mut partials)?;

    // Optionally swap R/L
    partials = partials
        .into_par_iter()
        .flat_map(|partial| [partial.transform_by(Mat4::refl(X)), partial])
        .collect();

    cleanup_and_display_solutions("stage 1", &mut partials, false);

    println!("Stage 2: Left");
    Iddfs::<Stage2, _>::new(Stage2::TWISTS, |s| s.is_target_solved(), 1..=6)
        .with_prune(|s, d| s2_prune.query_should_prune(s.into(), d))
        .iddfs_extend(&mut partials)?;
    cleanup_and_display_solutions("stage 2", &mut partials, false);

    println!("Stage 3: Counts");
    Iddfs::<Stage3, _>::new(Stage3::TWISTS, |s| s.is_target_solved(), 1..=4)
        .iddfs_extend(&mut partials)?;
    cleanup_and_display_solutions("stage 3", &mut partials, false);

    println!("Stage 4.1: Pre-domino (2x2x2)");
    Iddfs::<Stage4, _>::new(Stage4::TWISTS, |s| s.is_222_target_solved(), 1..=4)
        .iddfs_extend(&mut partials)?;
    cleanup_and_display_solutions("stage 4.1", &mut partials, false);

    println!("Stage 4.2: Pre-domino (2x2x3)");
    Iddfs::<Stage4, _>::new(Stage4::TWISTS, |s| s.is_223_target_solved(), 1..=4)
        .iddfs_extend(&mut partials)?;
    cleanup_and_display_solutions("stage 4.2", &mut partials, false);

    println!("Stage 4.3: Pre-domino (7 blocks)");
    Iddfs::<Stage4, _>::new(Stage4::TWISTS, |s| s.is_311_target_solved(7), 1..=4)
        .iddfs_extend(&mut partials)?;
    cleanup_and_display_solutions("stage 4.3", &mut partials, false);

    println!("Stage 4.4: Pre-domino (8 blocks)");
    Iddfs::<Stage4, _>::new(Stage4::TWISTS, |s| s.is_311_target_solved(8), 1..=4)
        .iddfs_extend(&mut partials)?;
    cleanup_and_display_solutions("stage 4.4", &mut partials, false);

    println!("Stage 4: Pre-domino");
    let target = Stage4::target();
    Iddfs::<Stage4, _>::new(Stage4::TWISTS, |s| s.is_target_solved(&target), 1..=7)
        .with_prune(|s, d| s4_prune.query_should_prune(s.key(), d))
        .iddfs_extend(&mut partials)?;
    cleanup_and_display_solutions("stage 4", &mut partials, true);

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
pub struct Iddfs<S, SF, PF = fn(S, u8) -> bool> {
    twist_subset: Vec<Twist>,
    is_solved: SF,
    should_prune: PF,
    depth_range: RangeInclusive<u8>,
    _marker: PhantomData<S>,
}

impl<S: Stage, SF> Iddfs<S, SF> {
    pub fn new(twist_subset: TwistSet, is_solved: SF, depth_range: RangeInclusive<u8>) -> Self
    where
        S: Stage,
        SF: Sync + Fn(S) -> bool,
    {
        Self {
            twist_subset: twist_subset.to_vec(),
            is_solved,
            should_prune: |_, _| false,
            depth_range,
            _marker: PhantomData,
        }
    }
}

impl<S: Stage, SF, PF> Iddfs<S, SF, PF> {
    pub fn with_prune<PF2>(self, should_prune: PF2) -> Iddfs<S, SF, PF2>
    where
        PF2: Sync + Fn(S, u8) -> bool,
    {
        Iddfs {
            twist_subset: self.twist_subset,
            is_solved: self.is_solved,
            should_prune,
            depth_range: self.depth_range,
            _marker: PhantomData,
        }
    }

    /// Extends each partial using the minimum search depth necessary. Returns
    /// `Ok` if successful, or `Err` if unsuccessful.
    pub fn iddfs_extend(&self, partials: &mut Vec<Partial>) -> Result<(), NoSolution>
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
                        .map(|new_segment| partial.push_segment(&new_segment))
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
    fn dfs(
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
