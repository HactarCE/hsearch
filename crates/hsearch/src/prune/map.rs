use std::collections::VecDeque;
use std::io::{BufRead, Write};

use bitbuffer::{BitReadBuffer, BitReadStream, BitWriteStream, LittleEndian};
use itertools::Itertools;

use crate::{HashMap, StageKeyU128};

const DEPTH_BITS: usize = 4;

#[derive(Debug, PartialEq, Eq)]
pub struct PruningMap {
    map: HashMap<u128, u8>,
    max_depth: u8,
}

impl PruningMap {
    /// Returns the depth of the search that generated the purning map.
    pub fn depth(&self) -> u8 {
        self.max_depth
    }

    /// Returns whether the given branch should be pruned.
    pub fn query_should_prune(&self, key: u128, remaining_search_depth: u8) -> bool {
        remaining_search_depth <= self.max_depth
            && self
                .map
                .get(&key)
                .is_none_or(|&d| remaining_search_depth < d)
    }

    pub fn query(&self, key: u128) -> Option<u8> {
        self.map.get(&key).copied()
    }

    pub fn load_or_generate<S: StageKeyU128>(max_depth: u8, filename: &str) -> Self {
        assert!(max_depth < 1 << DEPTH_BITS, "max_depth exceeds DEPTH_BITS");
        let filename = format!("{filename}_depth{max_depth}.bin");
        if std::fs::exists(&filename).unwrap_or(false) {
            print!("Loading pruning table {filename} ... ");
            std::io::stdout().flush().unwrap();
            let t = std::time::Instant::now();
            let this = Self::deserialize(max_depth, &std::fs::read(&filename).unwrap()).unwrap();
            assert_eq!(
                S::init().len(),
                this.map.values().filter(|&&v| v == 0).count(),
            );
            println!("done in {:.3?}", t.elapsed());
            this
        } else {
            println!("Missing pruning table {filename}; generating ...");
            let t = std::time::Instant::now();
            let this = Self::new::<S>(max_depth);
            let dur = t.elapsed();
            println!("Generated pruning table in {dur:.3?}. Serializing ...");
            let serialized = this.serialize();
            println!(
                "Pruning table file is {} bytes ({} entries). Press enter to save.",
                serialized.len(),
                this.map.len(),
            );
            std::io::stdin()
                .lock()
                .read_line(&mut String::new())
                .unwrap();
            print!("Saving pruning table to {filename} ... ");
            std::io::stdout().flush().unwrap();
            std::fs::write(&filename, &serialized).unwrap();
            println!("done");
            this
        }
    }

    pub fn new<S: StageKeyU128>(max_depth: u8) -> Self {
        let mut queue = VecDeque::new();
        let mut map = HashMap::default();
        for state in S::init() {
            map.insert(state.key(), 0);
            queue.push_back((state, 0));
        }

        let twists = S::PRUNING_MAP_TWISTS.to_vec();
        while let Some((state, depth)) = queue.pop_front() {
            let new_depth = depth + 1;
            for &twist in &twists {
                if let Some(new_state) = state.do_twist(twist)
                    && let std::collections::hash_map::Entry::Vacant(e) = map.entry(new_state.key())
                {
                    e.insert(new_depth);
                    if new_depth < max_depth {
                        queue.push_back((new_state, new_depth));
                    }
                }
            }
        }
        assert_eq!(S::init().len(), map.values().filter(|&&v| v == 0).count());

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
    map: &HashMap<u128, u8>,
    buf: &mut BitWriteStream<'_, LittleEndian>,
) -> bitbuffer::Result<()> {
    buf.write_int(map.len(), 64)?;
    for (&k, &v) in map.iter().sorted() {
        buf.write_int(k, 128)?;
        buf.write_int(v, DEPTH_BITS)?;
    }
    Ok(())
}

fn deser_from_buf(
    buf: &mut BitReadStream<'_, LittleEndian>,
) -> bitbuffer::Result<HashMap<u128, u8>> {
    let mut ret = HashMap::default();
    let entry_count = buf.read_int::<u128>(64)?;
    for _ in 0..entry_count {
        let key = buf.read_int::<u128>(128)?;
        let value = buf.read_int::<u8>(DEPTH_BITS)?;
        ret.insert(key, value);
    }
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{Stage, Stage4, parse_twists};

    #[test]
    fn test_pruning_map_ser_deser() {
        for depth in 1..=2 {
            let pruning_map = PruningMap::new::<Stage4>(depth);
            let serialized = pruning_map.serialize();
            let deserialized = PruningMap::deserialize(depth, &serialized).unwrap();
            assert_eq!(deserialized, pruning_map);
        }
    }

    #[test]
    fn test_pruning_map_determinism() {
        let map1 = PruningMap::new::<Stage4>(2);
        let map2 = PruningMap::new::<Stage4>(2);
        assert_eq!(map1, map2);
    }

    #[test]
    fn test_stage4_pruning_map() {
        let pruning_map = PruningMap::new::<Stage4>(2);
        let mut state = Stage4::default();
        assert_eq!(Some(&0), pruning_map.map.get(&state.key()));
        state = state.do_twists(parse_twists("FR")).unwrap();
        assert_eq!(Some(&1), pruning_map.map.get(&state.key()));
        state = state.do_twists(parse_twists("OF")).unwrap();
        assert_eq!(Some(&2), pruning_map.map.get(&state.key()));
        state = state.do_twists(parse_twists("FR")).unwrap();
        assert_eq!(Some(&3), pruning_map.map.get(&state.key()));
    }
}
