use std::io::{Read, Write};
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
    s1_ppsro: LazyLock::new(|| PruningTrie::load_or_generate::<Stage1>(3, "s1_ppsro")),
    s4_psio: LazyLock::new(|| PruningMap::load_or_generate::<Stage4>(9, "s4_psio")),
};

fn thread_local_bump_allocator() -> &'static bumpalo::Bump {
    thread_local! {
        static ALLOC: &'static bumpalo::Bump = Box::leak(Box::new(bumpalo::Bump::new()));
    }

    ALLOC.with(|a| *a)
}

fn compress(uncompressed_data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut compressed = vec![];
    let mut encoder = flate2::write::GzEncoder::new(&mut compressed, flate2::Compression::fast());
    encoder.write_all(uncompressed_data)?;
    encoder.finish()?;
    Ok(compressed)
}

fn read_and_uncompress(filename: impl AsRef<std::path::Path>) -> std::io::Result<Vec<u8>> {
    let reader = std::io::BufReader::new(std::fs::File::open(&filename)?);
    let mut uncompressed = vec![];
    flate2::read::GzDecoder::new(reader).read_to_end(&mut uncompressed)?;
    Ok(uncompressed)
}
