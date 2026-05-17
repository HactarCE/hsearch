use std::{collections::HashMap, hash::Hash, io::BufRead, sync::LazyLock};

use bitbuffer::{BitReadBuffer, BitReadStream, BitWriteStream, LittleEndian};
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::{
    Twist,
    canonical::PrevTwists,
    stages::{Stage, Stage1, SubsetMaskStage},
};

pub struct PruningTables {
    pub s1_pps: LazyLock<SubsetTrie>,
}

impl PruningTables {
    pub const S1_PPS_PRUNE_DEPTH: u8 = 3;
}

pub static PRUNING_TABLES: PruningTables = PruningTables {
    s1_pps: LazyLock::new(|| {
        crate::prune::SubsetTrie::make_or_load_pruning_table::<Stage1>(
            PruningTables::S1_PPS_PRUNE_DEPTH,
            "s1_pps",
        )
    }),
};

const BRANCHING_BITS: u32 = 4;
const BRANCHING_FACTOR: usize = 1 << BRANCHING_BITS;

/// Pruning table using a trie that requires the lookup key to have a subset of
/// the bits of the entry key. All matching entries are scanned and the one with
/// the lowest value is returned.
#[derive(Debug, Default, PartialEq, Eq)]
pub enum SubsetTrie {
    #[default]
    Empty, // solution unknown
    Terminal(u8),                                // move count
    Branch(Box<[SubsetTrie; BRANCHING_FACTOR]>), // index = bits that must be 1
}

impl SubsetTrie {
    pub fn lookup(&self, bits: u128) -> Option<u8> {
        match self {
            SubsetTrie::Empty => None,
            SubsetTrie::Terminal(move_count) => Some(*move_count),
            SubsetTrie::Branch(b) => {
                let next_bits = bits >> BRANCHING_BITS;
                b.iter()
                    .enumerate()
                    .filter(|(i, _b)| bits & *i as u128 == 0)
                    .filter_map(|(_i, b)| b.lookup(next_bits))
                    .min()
            }
        }
    }

    /// Inserts an entry if the new value is less than the old value.
    fn insert_if_better(&mut self, key: u128, bits_remaining: u32, value: u8) {
        if bits_remaining == 0 {
            if let Self::Terminal(old_value) = self {
                *old_value = std::cmp::min(*old_value, value);
            } else {
                *self = Self::Terminal(value);
            }
        } else {
            let mut b = match self {
                SubsetTrie::Empty => Box::new(std::array::from_fn(|_| Self::Empty)),
                SubsetTrie::Terminal(_) => return,
                SubsetTrie::Branch(b) => std::mem::take(b),
            };
            let index_mask = (1 << BRANCHING_BITS) - 1;
            let was_inserted = b[key as usize & index_mask].insert_if_better(
                key >> BRANCHING_BITS,
                bits_remaining.saturating_sub(BRANCHING_BITS),
                value,
            );
            *self = SubsetTrie::Branch(b);
            was_inserted
        }
    }

    pub fn new<S: SubsetMaskStage>(max_depth: u8) -> Self {
        let total_bits = S::SUBSET_TRIE_KEY_BITS;
        let init_mask = S::subset_trie_target();
        let entry_maps: Vec<HashMap<u128, u8>> = Twist::iter()
            .par_bridge()
            .map(|first_twist| {
                let mut entries = HashMap::new();
                for depth in 1..=max_depth {
                    let mut queue = vec![(
                        init_mask.do_twist(first_twist),
                        1,
                        PrevTwists::new().do_twist(first_twist).unwrap(),
                    )];
                    entries.insert(init_mask.do_twist(first_twist).subset_trie_key(), 1);
                    while let Some((state, d, prev_twists)) = queue.pop() {
                        let d = d + 1;
                        for twist in Twist::iter() {
                            if let Some(new_prev_twists) = prev_twists.do_twist(twist) {
                                let new_state = state.do_twist(twist);
                                match entries.entry(new_state.subset_trie_key()) {
                                    std::collections::hash_map::Entry::Occupied(mut e) => {
                                        if *e.get() > d {
                                            e.insert(d);
                                        } else if *e.get() < d {
                                            continue;
                                        }
                                    }
                                    std::collections::hash_map::Entry::Vacant(e) => {
                                        e.insert(d);
                                    }
                                }
                                if d < depth {
                                    queue.push((new_state, d, new_prev_twists));
                                }
                            }
                        }
                    }
                }
                entries
            })
            .collect();
        let mut ret = SubsetTrie::Empty;
        ret.insert_if_better(init_mask.subset_trie_key(), total_bits, 0);
        let mut new_hashmap = HashMap::new();
        for map in entry_maps {
            for (k, v) in map {
                match new_hashmap.entry(k) {
                    std::collections::hash_map::Entry::Occupied(mut e) => {
                        e.insert(std::cmp::min(*e.get(), v));
                    }
                    std::collections::hash_map::Entry::Vacant(e) => {
                        e.insert(v);
                    }
                }
            }
        }
        for (k, v) in new_hashmap {
            ret.insert_if_better(k, total_bits, v);
        }
        ret
    }

    pub fn make_or_load_pruning_table<S: SubsetMaskStage>(max_depth: u8, filename: &str) -> Self {
        let filename = format!("{filename}_depth{max_depth}.bin");
        if std::fs::exists(&filename).unwrap_or(false) {
            println!("Loading pruning table {filename}");
            let ret = Self::load_from_file::<S>(&filename, S::SUBSET_TRIE_KEY_BITS);
            println!("Done loading pruning table {filename}");
            ret
        } else {
            println!("Missing pruning table {filename}; generating ...");
            let pruning_table = Self::new::<S>(max_depth);
            let dur = t.elapsed();
            println!("Generated pruning table in {dur:?}. Serializing ...");
            let serialized = pruning_table.serialize();
            println!(
                "Pruning table file is {} bytes. Press enter to save.",
                serialized.len()
            );
            std::io::stdin()
                .lock()
                .read_line(&mut String::new())
                .unwrap();
            println!("Saving pruning table to {filename} ...");
            std::fs::write(&filename, &serialized).unwrap();
            println!("Done saving pruning table {filename}");
            pruning_table
        }
    }

    fn save_to_file(&self, filename: &str) {
        std::fs::write(filename, &self.serialize()).unwrap();
    }

    fn load_from_file<S: SubsetMaskStage>(filename: &str, key_bits: u32) -> Self {
        Self::deserialize(&std::fs::read(filename).unwrap(), key_bits)
    }

    fn serialize(&self) -> Vec<u8> {
        let mut buf = vec![];
        self.ser_to_buf(&mut BitWriteStream::new(&mut buf, LittleEndian));
        buf
    }

    fn deserialize(buf: &[u8], key_bits: u32) -> Self {
        Self::deser_from_buf(
            &mut BitReadStream::new(BitReadBuffer::new(&buf, LittleEndian)),
            key_bits,
        )
    }

    fn ser_to_buf(&self, buf: &mut BitWriteStream<'_, LittleEndian>) {
        match self {
            SubsetTrie::Empty => (), // serialize nothing
            SubsetTrie::Terminal(t) => buf.write_int(*t, 3).unwrap(), // 3 bits
            SubsetTrie::Branch(b) => {
                let mask = crate::util::collect_bits(
                    b.iter().map(|child| !matches!(child, SubsetTrie::Empty)),
                );
                buf.write_int(mask, BRANCHING_FACTOR).unwrap();
                for child in &**b {
                    child.ser_to_buf(buf);
                }
            }
        }
    }

    fn deser_from_buf(buf: &mut BitReadStream<'_, LittleEndian>, key_bits_remaining: u32) -> Self {
        if key_bits_remaining == 0 {
            SubsetTrie::Terminal(buf.read_int(3).unwrap())
        } else {
            let mask: u64 = buf.read_int(BRANCHING_FACTOR).unwrap();
            SubsetTrie::Branch(Box::new(std::array::from_fn(|i| {
                if (mask & (1 << i)) != 0 {
                    Self::deser_from_buf(buf, key_bits_remaining.saturating_sub(BRANCHING_BITS))
                } else {
                    SubsetTrie::Empty
                }
            })))
        }
    }
}

pub struct PruningTable<K>(HashMap<K, u8>);

impl<K: Copy + Eq + Hash> PruningTable<K> {
    pub fn make_pruning_table<S: Stage>(
        solved: S,
        get_key: impl Fn(S) -> K,
        max_depth: u8,
    ) -> Self {
        let mut ret = HashMap::new();
        ret.insert(get_key(solved), 0);
        for depth in 1..=max_depth {
            let mut queue = vec![(solved, 0)];
            while let Some((state, d)) = queue.pop() {
                let d = d + 1;
                for twist in Twist::iter() {
                    let new_state = state.do_twist(twist);
                    if d == depth {
                        ret.entry(get_key(new_state)).or_insert(d);
                    } else {
                        queue.push((new_state, d));
                    }
                }
            }
        }
        Self(ret)
    }
}

impl PruningTable<u32> {
    pub fn make_s1_e_pruning_table(max_depth: u8) -> Self {
        let mut ret = HashMap::new();
        let init = Stage1::default();
        ret.insert(init.e_p, 0);
        for depth in 1..=max_depth {
            let mut queue = vec![(init, 0)];
            while let Some((state, d)) = queue.pop() {
                let d = d + 1;
                for twist in Twist::iter() {
                    let new_state = state.do_twist(twist);
                    if d == depth {
                        ret.entry(new_state.e_p).or_insert(d);
                    } else {
                        queue.push((new_state, d));
                    }
                }
            }
        }
        Self(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pruning_trie_ser_deser() {
        let pruning_trie = SubsetTrie::new::<Stage1>(2);
        let serialized = pruning_trie.serialize();
        let deserialized = SubsetTrie::deserialize(&serialized, Stage1::SUBSET_TRIE_KEY_BITS);
        assert_eq!(deserialized, pruning_trie);
    }
}
