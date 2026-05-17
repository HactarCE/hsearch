use std::{collections::HashMap, hash::Hash, io::BufRead, sync::LazyLock};

use bitbuffer::{BitReadBuffer, BitReadStream, BitWriteStream, LittleEndian};
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::{
    Twist,
    canonical::PrevTwists,
    stages::{Stage, Stage1, SubsetMaskStage},
};

const DEPTH_BITS: usize = 3;

pub struct PruningTables {
    pub s1_pps: LazyLock<SubsetTrie>,
}

impl PruningTables {
    pub const S1_PPS_PRUNE_DEPTH: u8 = 4;
}

pub static PRUNING_TABLES: PruningTables = PruningTables {
    s1_pps: LazyLock::new(|| {
        crate::prune::SubsetTrie::make_or_load_pruning_table::<Stage1>(
            PruningTables::S1_PPS_PRUNE_DEPTH,
            "s1_pps",
        )
    }),
};

/// Pruning table using a path-compressed trie that requires the lookup key to
/// have a subset of the bits of the entry key. All matching entries are scanned
/// and the one with the lowest value is returned.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct SubsetTrie {
    /// Number of bits in `mask`.
    mask_len: u8,
    /// Mask that is required by `inner`.
    mask: u128,

    contents: SubsetTrieContents,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum SubsetTrieContents {
    #[default]
    Empty,
    Value(u8),
    Branch(Box<[SubsetTrie; 2]>),
}

impl SubsetTrieContents {
    fn query(&self, query_key: u128) -> Option<u8> {
        match self {
            SubsetTrieContents::Empty => None,
            SubsetTrieContents::Value(v) => Some(*v),
            SubsetTrieContents::Branch(children) => children
                .iter()
                .enumerate()
                .filter(|(i, _child)| query_key as usize & i == 0)
                .filter_map(|(_i, child)| child.query(query_key >> 1))
                .min(),
        }
    }

    /// Inserts an entry if it is less than the existing entry.
    fn insert(&mut self, entry_key: u128, key_bits_remaining: u8, new_value: u8) {
        match self {
            SubsetTrieContents::Empty => {
                *self = SubsetTrieContents::Value(new_value);
            }
            SubsetTrieContents::Value(old_value) => {
                if new_value < *old_value {
                    *old_value = new_value
                }
            }
            SubsetTrieContents::Branch(b) => {
                b[entry_key as usize & 1].insert(entry_key >> 1, key_bits_remaining, new_value);
            }
        }
    }
}

impl SubsetTrie {
    pub fn query(&self, query_key: u128) -> Option<u8> {
        if query_key & self.mask == 0 {
            self.contents.query(query_key >> self.mask_len)
        } else {
            None
        }
    }

    /// Inserts an entry if it is less than the existing entry.
    fn insert(&mut self, entry_key: u128, key_bits_remaining: u8, new_value: u8) {
        let shared_bits = (entry_key ^ self.mask).trailing_zeros() as u8;
        if shared_bits >= self.mask_len {
            self.contents
                .insert(entry_key >> self.mask_len, key_bits_remaining, new_value);
        } else {
            let old_child_branch_bit = (self.mask >> shared_bits) & 1;
            let old_child = SubsetTrie {
                mask_len: self.mask_len - shared_bits - 1,
                mask: self.mask >> (shared_bits + 1),
                contents: std::mem::take(&mut self.contents),
            };
            let new_child = SubsetTrie {
                mask_len: key_bits_remaining - shared_bits - 1,
                mask: entry_key >> (shared_bits + 1),
                contents: SubsetTrieContents::Value(new_value),
            };
            *self = SubsetTrie {
                mask_len: shared_bits,
                mask: self.mask & ((1 << shared_bits) - 1),
                contents: SubsetTrieContents::Branch(Box::new(if old_child_branch_bit == 0 {
                    [old_child, new_child]
                } else {
                    [new_child, old_child]
                })),
            };
        }
    }

    pub fn with_single_entry(key: u128, key_bits: u8, value: u8) -> Self {
        Self {
            mask_len: key_bits,
            mask: key,
            contents: SubsetTrieContents::Value(value),
        }
    }

    pub fn new<S: SubsetMaskStage>(max_depth: u8) -> Self {
        assert!(max_depth < ((1 << DEPTH_BITS) - 1));

        let total_bits = S::SUBSET_TRIE_KEY_BITS as u8;
        let init_mask = S::subset_trie_target();
        let t = std::time::Instant::now();
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
                    if depth <= 1 {
                        continue;
                    }
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
        let entry_count_estimate: usize = entry_maps.iter().map(|m| m.len()).sum();
        println!(
            "Generated pruning table contents in {:?} (~{} entries)",
            t.elapsed(),
            entry_count_estimate,
        );
        let mut ret = SubsetTrie::with_single_entry(init_mask.subset_trie_key(), total_bits, 0);
        let mut new_hashmap = HashMap::new();
        println!("Deduplicating entries ...");
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
        let entry_count = new_hashmap.len();
        println!("Assembling subset trie with {entry_count} entries ...");
        for (i, (&k, &v)) in new_hashmap.iter().enumerate() {
            ret.insert(k, total_bits, v);
            if i % 1_000_000 == 0 && i > 0 {
                println!("  done {i}/{entry_count}");
            }
        }
        ret
    }

    pub fn make_or_load_pruning_table<S: SubsetMaskStage>(max_depth: u8, filename: &str) -> Self {
        let filename = format!("{filename}_depth{max_depth}.bin");
        if std::fs::exists(&filename).unwrap_or(false) {
            println!("Loading pruning table {filename}");
            let ret = Self::load_from_file(&filename);
            println!("Done loading pruning table {filename}");
            ret
        } else {
            println!("Missing pruning table {filename}; generating ...");
            let t = std::time::Instant::now();
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

    fn load_from_file(filename: &str) -> Self {
        Self::deserialize(&std::fs::read(filename).unwrap()).unwrap()
    }

    fn serialize(&self) -> Vec<u8> {
        let mut buf = vec![];
        self.ser_to_buf(&mut BitWriteStream::new(&mut buf, LittleEndian));
        buf
    }

    fn deserialize(buf: &[u8]) -> bitbuffer::Result<Self> {
        Self::deser_from_buf(&mut BitReadStream::new(BitReadBuffer::new(
            &buf,
            LittleEndian,
        )))
    }

    fn ser_to_buf(&self, buf: &mut BitWriteStream<'_, LittleEndian>) -> bitbuffer::Result<()> {
        let Self {
            mask_len,
            mask,
            contents,
        } = self;
        buf.write_int(*mask_len, 8)?;
        buf.write_int(*mask, *mask_len as usize)?;
        match contents {
            SubsetTrieContents::Empty => {
                buf.write_bool(false)?;
                buf.write_int(0, DEPTH_BITS)?;
            }
            SubsetTrieContents::Value(v) => {
                buf.write_bool(false)?;
                buf.write_int(*v + 1, DEPTH_BITS)?;
            }
            SubsetTrieContents::Branch(children) => {
                buf.write_bool(true)?;
                children[0].ser_to_buf(buf)?;
                children[1].ser_to_buf(buf)?;
            }
        }
        Ok(())
    }

    fn deser_from_buf(buf: &mut BitReadStream<'_, LittleEndian>) -> bitbuffer::Result<Self> {
        let mask_len = buf.read_int::<u8>(8)?;
        let mask = buf.read_int::<u128>(mask_len as usize)?;
        let contents = if buf.read_bool()? {
            SubsetTrieContents::Branch(Box::new([
                Self::deser_from_buf(buf)?,
                Self::deser_from_buf(buf)?,
            ]))
        } else {
            match buf.read_int::<u8>(DEPTH_BITS)?.checked_sub(1) {
                Some(v) => SubsetTrieContents::Value(v),
                None => SubsetTrieContents::Empty,
            }
        };
        Ok(Self {
            mask_len,
            mask,
            contents,
        })
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
        let deserialized = SubsetTrie::deserialize(&serialized).unwrap();
        assert_eq!(deserialized, pruning_trie);
    }
}
