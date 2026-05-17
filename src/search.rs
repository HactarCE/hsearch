use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::canonical::PrevTwists;
use crate::prelude::*;
use crate::prune::{PRUNING_TABLES, PruningTables};
use crate::stages::*;
use crate::util::twists_to_string;

pub fn solve(scramble: Vec<Twist>) -> Vec<Twist> {
    let s1_pps_prune = &*PRUNING_TABLES.s1_pps;

    let init_options = itertools::iproduct!(
        Axis::ALL.map(|src| (src, Mat4::rot(src, W))),
        [R, L, U, D, F, B].map(|f| (f, f.mat4_to(F)))
    )
    .map(|((a, m1), (b, m2))| (format!("{a},{b}"), m2 * m1))
    .map(|(option_name, m)| {
        let transformed_scramble = scramble.iter().map(|t| t.transform_by(m)).collect_vec();
        (
            format!(
                "{option_name} ... {}",
                twists_to_string(&transformed_scramble),
            ),
            Stage1::with_setup(&transformed_scramble),
        )
    })
    .collect_vec();
    let solutions = iddfs(
        &init_options,
        |state, remaining_search_depth| {
            remaining_search_depth <= PruningTables::S1_PPS_PRUNE_DEPTH
                && s1_pps_prune.query_should_prune(state.subset_trie_key(), remaining_search_depth)
        },
        6,
    );
    solutions.into_iter().next().unwrap_or_default()
}

pub fn unwrap_iddfs<S: Stage>(
    init_options: &[(String, S)],
    should_prune: impl Sync + Fn(S, u8) -> bool,
    max_depth: u8,
) -> Vec<Twist> {
    let solutions = iddfs(init_options, should_prune, max_depth);
    println!("Found {} solutions", solutions.len());
    solutions.into_iter().next().expect("no solution found")
}

/// Iterative-deepening depth-first search.
pub fn iddfs<S: Stage>(
    init_options: &[(String, S)],
    should_prune: impl Sync + Fn(S, u8) -> bool,
    max_depth: u8,
) -> Vec<Vec<Twist>> {
    for depth in 0..=max_depth {
        // println!("Searching at depth {depth} ...");
        let solutions: Vec<(&str, Vec<Vec<Twist>>)> = init_options
            .par_iter()
            .map(|(option_name, init)| {
                let mut solutions = vec![];
                dfs(
                    *init,
                    PrevTwists::new(),
                    &should_prune,
                    depth,
                    &mut vec![],
                    &mut solutions,
                );
                (option_name.as_str(), solutions)
            })
            .collect();
        let count: usize = solutions.iter().map(|(_, list)| list.len()).sum();
        if count > 0 {
            println!("Found {count} solutions at depth {depth}:");
            // for (option_name, solution_list) in &solutions {
            //     for sol in solution_list {
            //         println!(
            //             "  {option_name}        {}",
            //             crate::util::twists_to_string(sol)
            //         );
            //     }
            // }
            return solutions
                .into_iter()
                .flat_map(|(_, sol_list)| sol_list)
                .collect();
        }
    }
    println!("Found 0 solutions");
    vec![] // no solutions :(
}

/// Depth-first search.
pub fn dfs<S: Stage>(
    state: S,
    prev_twists: PrevTwists,
    should_prune: &impl Fn(S, u8) -> bool,
    remaining_depth: u8,
    solution_buffer: &mut Vec<Twist>,
    solutions: &mut Vec<Vec<Twist>>,
) {
    if state.is_solved() {
        solutions.push(solution_buffer.clone());
        return;
    }
    if remaining_depth == 0 || should_prune(state, remaining_depth) {
        return;
    }

    for twist in Twist::iter() {
        let Some(new_prev_twists) = prev_twists.do_twist(twist) else {
            continue;
        };
        solution_buffer.push(twist);
        dfs(
            state.do_twist(twist),
            new_prev_twists,
            should_prune,
            remaining_depth - 1,
            solution_buffer,
            solutions,
        );
        solution_buffer.pop();
    }
}
