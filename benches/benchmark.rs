#![allow(unused_crate_dependencies)]

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use hsearch::{SCRAMBLE_LEN, prelude::*};
use itertools::Itertools;
use rand::{
    SeedableRng,
    seq::{IndexedRandom, IteratorRandom},
};
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    use hsearch::stages::*;

    // println!("Hello, world!");
    let mut g = c.benchmark_group("stage1_pruning_trie_lookup");

    let mut rng = rand::rngs::StdRng::seed_from_u64(0);

    let init_state = Stage1::with_setup(
        &Twist::iter()
            .filter(|t| [I, O, F].contains(&t.facet()))
            .collect_vec()
            .choose_iter(&mut rng)
            .unwrap()
            .take(SCRAMBLE_LEN)
            .copied()
            .collect_vec(),
    );
    assert!(init_state.is_solved());

    for prune_depth in [4] {
        let pruning_trie = PruningTrie::load_or_generate::<Stage1>(prune_depth, "s1_ppsro");
        for distance_to_solved in [4, 10] {
            let input_states = (0..100)
                .map(|_| {
                    std::iter::from_fn(|| Twist::iter().choose(&mut rng))
                        .take(distance_to_solved)
                        .fold(init_state, Stage::do_twist)
                })
                .collect_vec();
            for remaining_search_depth in [1, 2, 3, 4] {
                let mut input_states_iter = input_states.iter().copied().cycle();
                let id = BenchmarkId::from_parameter(format!(
                    "p={prune_depth},d={distance_to_solved},s={remaining_search_depth}"
                ));
                g.bench_function(id, |b| {
                    b.iter(|| {
                        let s = black_box(input_states_iter.next().unwrap());
                        pruning_trie.query_should_prune(s.subset_trie_key(), remaining_search_depth)
                    });
                });
            }
        }
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
