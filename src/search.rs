use crate::canonical::PrevTwists;
use crate::prelude::*;
use crate::stages::*;

pub fn solve(scramble: Vec<Twist>) -> Vec<Twist> {
    let mut setup = scramble;
    let mut solution = vec![];

    let new_twists = &unwrap_iddfs(Stage1::with_setup(&setup), |s| s.is_step1_solved(), 3);
    setup.extend_from_slice(&new_twists);
    solution.extend_from_slice(&new_twists);
    println!("{}", crate::util::twists_to_string(&solution));

    let new_twists = &unwrap_iddfs(Stage1::with_setup(&setup), |s| s.is_step2_solved(), 3);
    setup.extend_from_slice(&new_twists);
    solution.extend_from_slice(&new_twists);
    println!("{}", crate::util::twists_to_string(&solution));

    solution.extend(&unwrap_iddfs(
        Stage1::with_setup(&setup),
        |s| s.is_solved(),
        4,
    ));
    println!("{}", crate::util::twists_to_string(&solution));
    solution
}

pub fn unwrap_iddfs<S: Stage>(
    init: S,
    is_solved: impl Fn(S) -> bool,
    max_depth: usize,
) -> Vec<Twist> {
    iddfs(init, is_solved, max_depth)
        .into_iter()
        .next()
        .expect("no solution found")
}

/// Iterative-deepening depth-first search.
pub fn iddfs<S: Stage>(
    init: S,
    is_solved: impl Fn(S) -> bool,
    max_depth: usize,
) -> Vec<Vec<Twist>> {
    for depth in 0..=max_depth {
        // println!("Searching at depth {depth} ...");
        let mut solutions = vec![];
        dfs(
            init,
            PrevTwists::new(),
            &is_solved,
            depth,
            &mut vec![],
            &mut solutions,
        );
        if !solutions.is_empty() {
            // println!("Found {} solutions", solutions.len());
            return solutions;
        }
    }
    vec![] // no solutions :(
}

/// Depth-first search.
pub fn dfs<S: Stage>(
    state: S,
    prev_twists: PrevTwists,
    is_solved: &impl Fn(S) -> bool,
    remaining_depth: usize,
    solution_buffer: &mut Vec<Twist>,
    solutions: &mut Vec<Vec<Twist>>,
) {
    if is_solved(state) {
        solutions.push(solution_buffer.clone());
    }
    if remaining_depth == 0 {
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
            is_solved,
            remaining_depth - 1,
            solution_buffer,
            solutions,
        );
        if !solutions.is_empty() {
            return;
        }
        solution_buffer.pop();
    }
}
