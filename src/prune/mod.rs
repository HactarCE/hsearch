use std::sync::LazyLock;

use crate::stages::*;

mod map;
mod trie;

pub use map::PruningMap;
pub use trie::PruningTrie;

pub struct PruningTables {
    pub s1_ppsro: LazyLock<PruningTrie>,
    pub s4_psio: LazyLock<PruningMap>,
}

pub static PRUNING_TABLES: PruningTables = PruningTables {
    s1_ppsro: LazyLock::new(|| PruningTrie::load_or_generate::<Stage1>(4, "s1_ppsro")),
    s4_psio: LazyLock::new(|| PruningMap::load_or_generate::<Stage4>(9, "s4_psio")),
};

fn thread_local_bump_allocator() -> &'static bumpalo::Bump {
    thread_local! {
        static ALLOC: &'static bumpalo::Bump = Box::leak(Box::new(bumpalo::Bump::new()));
    }

    ALLOC.with(|a| *a)
}
