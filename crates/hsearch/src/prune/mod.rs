use std::sync::LazyLock;

use crate::stages::*;

mod map;
mod trie;
mod trie_key;

pub use map::PruningMap;
pub use trie::PruningTrie;
pub use trie_key::*;

pub struct PruningTables {
    pub s1_mid: LazyLock<PruningTrie<Stage1>>,
    pub s2_left: LazyLock<PruningTrie<Stage2>>,
}

pub static PRUNING_TABLES: PruningTables = PruningTables {
    s1_mid: LazyLock::new(|| {
        PruningTrie::<Stage1>::load_or_generate(&[Stage1::TARGET], Stage1::TWISTS, 4, "s1_mid")
    }),
    s2_left: LazyLock::new(|| {
        PruningTrie::<Stage2>::load_or_generate(&[Stage2::TARGET], Stage2::TWISTS, 4, "s1_left")
    }),
};

fn thread_local_bump_allocator() -> &'static bumpalo::Bump {
    thread_local! {
        static ALLOC: &'static bumpalo::Bump = Box::leak(Box::new(bumpalo::Bump::new()));
    }

    ALLOC.with(|a| *a)
}
