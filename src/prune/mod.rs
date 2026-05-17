use std::sync::LazyLock;

use crate::stages::*;

mod trie;

pub use trie::PruningTrie;

pub struct PruningTables {
    pub s1_pps: LazyLock<PruningTrie>,
}

pub static PRUNING_TABLES: PruningTables = PruningTables {
    s1_pps: LazyLock::new(|| PruningTrie::load_or_generate::<Stage1>(4, "s1_ppsro")),
};

fn thread_local_bump_allocator() -> &'static bumpalo::Bump {
    thread_local! {
        static ALLOC: &'static bumpalo::Bump = Box::leak(Box::new(bumpalo::Bump::new()));
    }

    ALLOC.with(|a| *a)
}
