#![allow(unused_crate_dependencies)]

use std::hint::black_box;

use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, BenchmarkId, Criterion, criterion_group, criterion_main};
use hsearch::SCRAMBLE_LEN;
use hsearch::prelude::*;
use hsearch::stages::*;
use itertools::Itertools;
use rand::SeedableRng;
use rand::seq::{IndexedRandom, IteratorRandom};

criterion_main!(benches);
criterion_group!(benches, criterion_benchmark);

fn criterion_benchmark(c: &mut Criterion) {
    bench_do_twist(c);
    bench_pruning_trie(c);
}

fn bench_do_twist(c: &mut Criterion) {
    fn bench_stage_do_twist<S: Stage>(g: &mut BenchmarkGroup<'_, WallTime>, name: &str) {
        let allowed_twists = S::TWISTS.to_vec();
        let twist_sequence = {
            let mut twist_rng = rand::rngs::StdRng::seed_from_u64(1);
            (0..500)
                .map(|_| *allowed_twists.choose(&mut twist_rng).unwrap())
                .collect_vec()
        };

        g.bench_function(name, |b| {
            b.iter(|| {
                let state = black_box(S::default());
                black_box(twist_sequence.iter().copied().fold(state, S::do_twist))
            });
        });
    }

    let mut g = c.benchmark_group("do_twist");
    bench_stage_do_twist::<Stage1>(&mut g, "Stage1");
    bench_stage_do_twist::<Stage2>(&mut g, "Stage2");
    bench_stage_do_twist::<Stage3>(&mut g, "Stage3");
    g.finish();
}

fn bench_pruning_trie(c: &mut Criterion) {
    fn bench_stage_pruning_trie_lookup<S: SubsetMaskStage>(
        g: &mut BenchmarkGroup<'_, WallTime>,
        targets: &[S],
        prune_depth: u8,
        remaining_search_depth: u8,
        name: &str,
    ) {
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);
        let pruning_trie = PruningTrie::load_or_generate(targets, S::TWISTS, prune_depth, "s1_mid");
        for distance_to_solved in [4, 10] {
            let allowed_twists = S::TWISTS.to_vec();
            let input_states = (0..100)
                .map(|_| {
                    allowed_twists
                        .choose_iter(&mut rng)
                        .unwrap()
                        .copied()
                        .take(distance_to_solved)
                        .fold(S::default(), Stage::do_twist)
                })
                .collect_vec();
            let mut input_states_iter = input_states.iter().copied().cycle();
            let id = BenchmarkId::from_parameter(format!(
                "{name} p={prune_depth},d={distance_to_solved},s={remaining_search_depth}"
            ));
            g.bench_function(id, |b| {
                b.iter(|| {
                    let s = black_box(input_states_iter.next().unwrap());
                    pruning_trie.query_should_prune(s.into(), remaining_search_depth)
                });
            });
        }
    }

    let mut g = c.benchmark_group("stage1_pruning_trie_lookup");

    // bench_stage_pruning_trie_lookup(&mut g, &[Stage1::TARGET], 3, "Stage1");
    // bench_stage_pruning_trie_lookup(&mut g, &[Stage1::TARGET], 4, "Stage1");
    bench_stage_pruning_trie_lookup(&mut g, &[Stage2::TARGET], 3, 2, "Stage2");
    bench_stage_pruning_trie_lookup(&mut g, &[Stage2::TARGET], 4, 3, "Stage2");
}
