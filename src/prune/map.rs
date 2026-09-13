use std::collections::VecDeque;
use std::io::{BufRead, Write};

use bitbuffer::{BitReadBuffer, BitReadStream, BitWriteStream, LittleEndian};
use itertools::Itertools;
use rapidhash::HashMapExt;

use crate::{HashMap, StageKeyU64};

const DEPTH_BITS: usize = 4;

#[derive(Debug, PartialEq, Eq)]
pub struct PruningMap {
    map: HashMap<u64, u8>,
    max_depth: u8,
}

impl PruningMap {
    /// Returns the depth of the search that generated the purning map.
    pub fn depth(&self) -> u8 {
        self.max_depth
    }

    /// Returns whether the given branch should be pruned.
    pub fn query_should_prune(&self, key: u64, remaining_search_depth: u8) -> bool {
        remaining_search_depth <= self.max_depth
            && self
                .map
                .get(&key)
                .is_none_or(|&d| remaining_search_depth < d)
    }

    pub fn query(&self, key: u64) -> Option<u8> {
        self.map.get(&key).copied()
    }

    pub fn load_or_generate<S: StageKeyU64>(max_depth: u8, filename: &str) -> Self {
        assert!(max_depth < 1 << DEPTH_BITS, "max_depth exceeds DEPTH_BITS");
        let filename = format!("{filename}_depth{max_depth}.bin.gz");
        if std::fs::exists(&filename).unwrap_or(false) {
            print!("Loading pruning table {filename} ... ");
            std::io::stdout().flush().unwrap();
            let t = std::time::Instant::now();
            let serialized = super::read_and_uncompress(&filename).expect("decompression failed");
            let this = Self::deserialize(max_depth, &serialized).unwrap();
            assert_eq!(1, this.map.values().filter(|&&v| v == 0).count());
            println!("done in {:.3?}", t.elapsed());
            this
        } else {
            println!("Missing pruning table {filename}; generating ...");
            let t = std::time::Instant::now();
            let this = Self::new::<S>(max_depth);
            println!("Generated pruning table in {:.3?}", t.elapsed());

            print!("Serializing ... ");
            std::io::stdout().flush().unwrap();
            let t = std::time::Instant::now();
            let serialized = this.serialize();
            println!("done in {:.3?} ({} bytes)", t.elapsed(), serialized.len());
            print!("Compressing ... ");
            std::io::stdout().flush().unwrap();
            let t = std::time::Instant::now();
            let compressed = super::compress(&serialized).unwrap();
            println!("done in {:.3?} ({} bytes)", t.elapsed(), compressed.len());
            println!("Press enter to save.");
            std::io::stdin()
                .lock()
                .read_line(&mut String::new())
                .unwrap();
            println!("Saving pruning table to {filename} ...");
            std::fs::write(&filename, compressed).unwrap();
            println!("Done saving pruning table {filename}");
            this
        }
    }

    pub fn new<S: StageKeyU64>(max_depth: u8) -> Self {
        let init = S::default();

        let mut queue = VecDeque::new();
        queue.push_back((init, 0));

        let mut map = HashMap::new();
        map.insert(init.key(), 0);

        while let Some((state, depth)) = queue.pop_front() {
            let new_depth = depth + 1;
            for &twist in S::PRUNING_MAP_TWISTS {
                let new_state = state.do_twist(twist);
                if let std::collections::hash_map::Entry::Vacant(e) = map.entry(new_state.key()) {
                    e.insert(new_depth);
                    if new_depth < max_depth {
                        queue.push_back((new_state, new_depth));
                    }
                }
            }
        }
        assert_eq!(1, map.values().filter(|&&v| v == 0).count());

        Self { map, max_depth }
    }

    fn serialize(&self) -> Vec<u8> {
        let mut buf = vec![];
        ser_to_buf(&self.map, &mut BitWriteStream::new(&mut buf, LittleEndian)).unwrap();
        buf
    }

    fn deserialize(max_depth: u8, buf: &[u8]) -> bitbuffer::Result<Self> {
        let map = deser_from_buf(&mut BitReadStream::new(BitReadBuffer::new(
            buf,
            LittleEndian,
        )))?;
        Ok(Self { map, max_depth })
    }
}

fn ser_to_buf(
    map: &HashMap<u64, u8>,
    buf: &mut BitWriteStream<'_, LittleEndian>,
) -> bitbuffer::Result<()> {
    buf.write_int(map.len(), 64)?;
    for (&k, &v) in map.iter().sorted() {
        buf.write_int(k, 64)?;
        buf.write_int(v, DEPTH_BITS)?;
    }
    Ok(())
}

fn deser_from_buf(
    buf: &mut BitReadStream<'_, LittleEndian>,
) -> bitbuffer::Result<HashMap<u64, u8>> {
    let mut ret = HashMap::new();
    let entry_count = buf.read_int::<u64>(64)?;
    for _ in 0..entry_count {
        let key = buf.read_int::<u64>(64)?;
        let value = buf.read_int::<u8>(DEPTH_BITS)?;
        ret.insert(key, value);
    }
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    use crate::{Stage, parse_twists, stages::Stage4};

    #[test]
    fn test_pruning_trie_ser_deser() {
        for depth in 1..=4 {
            let pruning_map = PruningMap::new::<Stage4>(depth);
            let serialized = pruning_map.serialize();
            let deserialized = PruningMap::deserialize(depth, &serialized).unwrap();
            assert_eq!(deserialized, pruning_map);
        }
    }

    #[test]
    fn test_pruning_trie_determinism() {
        let map1 = PruningMap::new::<Stage4>(4);
        let map2 = PruningMap::new::<Stage4>(4);
        assert_eq!(map1, map2);
    }

    #[test]
    fn test_stage4_pruning_map() {
        let pruning_map = PruningMap::new::<Stage4>(4);
        let mut state = Stage4::default();
        assert_eq!(Some(&0), pruning_map.map.get(&state.key()));
        state = state.do_twists(parse_twists("FR"));
        assert_eq!(Some(&1), pruning_map.map.get(&state.key()));
        state = state.do_twists(parse_twists("OF"));
        assert_eq!(Some(&2), pruning_map.map.get(&state.key()));
        state = state.do_twists(parse_twists("FR"));
        assert_eq!(Some(&3), pruning_map.map.get(&state.key()));
    }
}
