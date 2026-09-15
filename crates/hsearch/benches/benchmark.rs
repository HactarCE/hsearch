#![allow(unused_crate_dependencies)]

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use hsearch::{SCRAMBLE_LEN, prelude::*, stages::*};
use itertools::Itertools;
use rand::{
    SeedableRng,
    seq::{IndexedRandom, IteratorRandom},
};
use std::hint::black_box;

criterion_main!(benches);
criterion_group!(benches, criterion_benchmark);

fn criterion_benchmark(c: &mut Criterion) {
    bench_do_twist(c);
    bench_pruning_trie(c);
}

fn bench_do_twist(c: &mut Criterion) {
    let mut g = c.benchmark_group("stage1_do_twist");
    let twist_sequence = {
        let mut twist_rng = rand::rngs::StdRng::seed_from_u64(1);
        (0..500)
            .map(|_| Twist::iter().choose(&mut twist_rng).unwrap())
            .collect_vec()
    };
    g.bench_function("Stage1", |b| {
        b.iter(|| {
            let state = black_box(Stage1::default());
            black_box(twist_sequence.iter().copied().fold(state, Stage1::do_twist))
        });
    });
    g.bench_function("V2Stage1", |b| {
        b.iter(|| {
            let state = black_box(V2Stage1::default());
            black_box(
                twist_sequence
                    .iter()
                    .copied()
                    .fold(state, V2Stage1::do_twist),
            )
        });
    });
    g.finish();

    let mut g = c.benchmark_group("stage2_do_twist");
    let twist_sequence = {
        let mut twist_rng = rand::rngs::StdRng::seed_from_u64(1);
        (0..500)
            .map(|_| {
                Stage2::TWISTS
                    .iter()
                    .copied()
                    .choose(&mut twist_rng)
                    .unwrap()
            })
            .collect_vec()
    };
    g.bench_function("Stage1", |b| {
        b.iter(|| {
            let state = black_box(Stage1::default());
            black_box(twist_sequence.iter().copied().fold(state, Stage1::do_twist))
        });
    });
    g.bench_function("Stage2", |b| {
        b.iter(|| {
            let state = black_box(Stage2::default());
            black_box(twist_sequence.iter().copied().fold(state, Stage2::do_twist))
        });
    });
    g.finish();
}

fn bench_pruning_trie(c: &mut Criterion) {
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
        let pruning_trie = PruningTrie::<Stage1>::load_or_generate(
            &[Stage1::TARGET],
            &Twist::ALL,
            prune_depth,
            "s1_ppsro",
        );
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
                        pruning_trie.query_should_prune(s.into(), remaining_search_depth)
                    });
                });
            }
        }
    }
}
