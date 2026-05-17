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
        |s| {
            if s.is_solved() {
                0
            } else {
                s1_pps_prune
                    .lookup(s.subset_trie_key())
                    .unwrap_or(PruningTables::S1_PPS_PRUNE_DEPTH + 1)
            }
        },
        6,
    );
    solutions.into_iter().next().unwrap_or_default()
}

pub fn unwrap_iddfs<S: Stage>(
    init_options: &[(String, S)],
    get_distance_lower_bound: impl Sync + Fn(S) -> u8,
    max_depth: u8,
) -> Vec<Twist> {
    let solutions = iddfs(init_options, get_distance_lower_bound, max_depth);
    println!("Found {} solutions", solutions.len());
    solutions.into_iter().next().expect("no solution found")
}

/// Iterative-deepening depth-first search.
pub fn iddfs<S: Stage>(
    init_options: &[(String, S)],
    get_distance_lower_bound: impl Sync + Fn(S) -> u8,
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
                    &get_distance_lower_bound,
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
            for (option_name, solution_list) in &solutions {
                for sol in solution_list {
                    println!(
                        "  {option_name}        {}",
                        crate::util::twists_to_string(sol)
                    );
                }
            }
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
    get_distance_lower_bound: &impl Fn(S) -> u8,
    remaining_depth: u8,
    solution_buffer: &mut Vec<Twist>,
    solutions: &mut Vec<Vec<Twist>>,
) {
    if state.is_solved() {
        solutions.push(solution_buffer.clone());
        return;
    } else {
        if get_distance_lower_bound(state) > remaining_depth {
            return; // prune
        }
    }

    if remaining_depth == 0 {
        return; // die
    }

    for twist in Twist::iter() {
        let Some(new_prev_twists) = prev_twists.do_twist(twist) else {
            continue;
        };
        solution_buffer.push(twist);
        dfs(
            state.do_twist(twist),
            new_prev_twists,
            get_distance_lower_bound,
            remaining_depth - 1,
            solution_buffer,
            solutions,
        );
        solution_buffer.pop();
        if !solutions.is_empty() {
            return;
        }
    }
}
