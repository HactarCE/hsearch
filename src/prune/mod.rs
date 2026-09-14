use std::sync::LazyLock;

use crate::{Twist, stages::*};

mod map;
mod trie;
mod trie_key;

pub use map::PruningMap;
pub use trie::PruningTrie;
pub use trie_key::*;

pub struct PruningTables {
    // pub s1_ppsro: LazyLock<PruningTrie<Stage1>>,
    // pub s4_psio: LazyLock<PruningMap>,
    pub v2_s1_sio_1: LazyLock<PruningTrie<V2Stage1>>,
    pub v2_s1_sio_2: LazyLock<PruningTrie<V2Stage1>>,
    pub v2_s1_sio_3: LazyLock<PruningTrie<V2Stage1>>,
    pub v2_s1_sio_4: LazyLock<PruningTrie<V2Stage1>>,
    pub v2_s1_sio_5: LazyLock<PruningTrie<V2Stage1>>,
    pub v2_s1_sio_6: LazyLock<PruningTrie<V2Stage1>>,
}

pub static PRUNING_TABLES: PruningTables = PruningTables {
    // s1_ppsro: LazyLock::new(|| {
    //     PruningTrie::<Stage1>::load_or_generate(Stage1::TARGET, 4, "s1_ppsro")
    // }),
    // s4_psio: LazyLock::new(|| PruningMap::load_or_generate::<Stage4>(9, "s4_psio")),
    v2_s1_sio_1: LazyLock::new(|| {
        PruningTrie::<V2Stage1>::load_or_generate(
            V2Stage1::TARGET1,
            &Twist::ALL,
            3,
            "v2_s1_sio_target1_rot",
        )
    }),
    v2_s1_sio_2: LazyLock::new(|| {
        PruningTrie::<V2Stage1>::load_or_generate(
            V2Stage1::TARGET2,
            &Twist::ALL,
            3,
            "v2_s1_sio_target2_rot",
        )
    }),
    v2_s1_sio_3: LazyLock::new(|| {
        PruningTrie::<V2Stage1>::load_or_generate(
            V2Stage1::TARGET3,
            &Twist::ALL,
            3,
            "v2_s1_sio_target3_rot",
        )
    }),
    v2_s1_sio_4: LazyLock::new(|| {
        PruningTrie::<V2Stage1>::load_or_generate(
            V2Stage1::TARGET4,
            &Twist::ALL,
            3,
            "v2_s1_sio_target4_rot",
        )
    }),
    v2_s1_sio_5: LazyLock::new(|| {
        PruningTrie::<V2Stage1>::load_or_generate(
            V2Stage1::TARGET5,
            &Twist::ALL,
            3,
            "v2_s1_sio_target5_rot",
        )
    }),
    v2_s1_sio_6: LazyLock::new(|| {
        PruningTrie::<V2Stage1>::load_or_generate(
            V2Stage1::TARGET6,
            &Twist::ALL,
            3,
            "v2_s1_sio_target6_rot",
        )
    }),
};

fn thread_local_bump_allocator() -> &'static bumpalo::Bump {
    thread_local! {
        static ALLOC: &'static bumpalo::Bump = Box::leak(Box::new(bumpalo::Bump::new()));
    }

    ALLOC.with(|a| *a)
}
