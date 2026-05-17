use std::ops::{Deref, DerefMut};
use std::{collections::HashMap, io::BufRead};

use bitbuffer::{BitReadBuffer, BitReadStream, BitWriteStream, LittleEndian};
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::{Twist, canonical::PrevTwists, stages::SubsetMaskStage};

const DEPTH_BITS: usize = 3;

#[derive(Debug)]
pub struct PruningTrie {
    root: TrieNode,
    max_depth: u8,
}

impl PruningTrie {
    /// Returns the depth of the search that generated the purning trie.
    pub fn depth(&self) -> u8 {
        self.max_depth
    }

    /// Returns whether the given branch should be pruned.
    pub fn query_should_prune(&self, query_key: u128, remaining_search_depth: u8) -> bool {
        remaining_search_depth <= self.max_depth
            && self
                .root
                .query_should_prune(query_key, remaining_search_depth)
    }

    /// Loads a pruning table from a file, or generates one if the file is
    /// missing.
    ///
    /// Prompts the user before saving a new file.
    pub fn load_or_generate<S: SubsetMaskStage>(max_depth: u8, filename: &str) -> Self {
        let filename = format!("{filename}_depth{max_depth}.bin");
        let root;
        if std::fs::exists(&filename).unwrap_or(false) {
            println!("Loading pruning table {filename}");
            root = TrieNode::deserialize(&std::fs::read(&filename).unwrap()).unwrap();
            println!("Done loading pruning table {filename}");
        } else {
            println!("Missing pruning table {filename}; generating ...");
            let t = std::time::Instant::now();
            root = TrieNode::new::<S>(max_depth);
            let dur = t.elapsed();
            println!("Generated pruning table in {dur:?}. Serializing ...");
            let serialized = root.serialize();
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
        }
        Self { root, max_depth }
    }
}

/// Pruning table using a path-compressed trie that requires the lookup key to
/// have a subset of the bits of the entry key. All matching entries are scanned
/// and the one with the lowest value is returned.
#[derive(Debug, Default, PartialEq, Eq)]
struct TrieNode {
    /// Number of bits in `mask`.
    mask_len: u8,
    /// Mask that is required by `inner`.
    mask: u128,
    /// Minimum lower bound among all descendants.
    lower_bound: u8,

    children: Option<&'static mut TrieChildren>,
}

impl TrieNode {
    fn query_should_prune(&self, query_key: u128, remaining_search_depth: u8) -> bool {
        if query_key & self.mask == 0 {
            self.lower_bound > remaining_search_depth
                || self.children.as_ref().is_some_and(|children| {
                    let child_bit = (query_key >> self.mask_len) & 1;
                    let child_key = (query_key >> self.mask_len) >> 1;
                    if child_bit == 0 {
                        children[0].query_should_prune(child_key, remaining_search_depth)
                            && children[1].query_should_prune(child_key, remaining_search_depth)
                    } else {
                        children[0].query_should_prune(child_key, remaining_search_depth)
                    }
                })
        } else {
            true
        }
    }

    /// Inserts an entry if it is less than the existing entry.
    fn insert(
        &mut self,
        alloc: &'static bumpalo::Bump,
        entry_key: u128,
        key_bits_remaining: u8,
        new_value: u8,
    ) {
        let shared_bits = (entry_key ^ self.mask).trailing_zeros() as u8;
        if shared_bits >= self.mask_len {
            if new_value < self.lower_bound {
                self.lower_bound = new_value;
            }
            match &mut self.children {
                Some(children) => {
                    let child_bit = (entry_key >> self.mask_len) & 1;
                    let child_key = (entry_key >> self.mask_len) >> 1;
                    let child_bits_remaining = key_bits_remaining - self.mask_len - 1;
                    children[child_bit as usize].insert(
                        alloc,
                        child_key,
                        child_bits_remaining,
                        new_value,
                    );
                }
                None => return,
            }
        } else {
            let old_child_branch_bit = (self.mask >> shared_bits) & 1;
            let old_child = TrieNode {
                mask_len: self.mask_len - shared_bits - 1,
                mask: self.mask >> (shared_bits + 1),
                lower_bound: self.lower_bound,
                children: self.children.take(),
            };
            let new_child = TrieNode {
                mask_len: key_bits_remaining - shared_bits - 1,
                mask: entry_key >> (shared_bits + 1),
                lower_bound: new_value,
                children: None,
            };
            *self = TrieNode {
                mask_len: shared_bits,
                mask: self.mask & ((1 << shared_bits) - 1),
                lower_bound: std::cmp::min(self.lower_bound, new_value),
                children: Some(alloc.alloc(TrieChildren(if old_child_branch_bit == 0 {
                    [old_child, new_child]
                } else {
                    [new_child, old_child]
                }))),
            };
        }
    }

    fn with_single_entry(key: u128, key_bits: u8, value: u8) -> Self {
        Self {
            mask_len: key_bits,
            mask: key,
            lower_bound: value,
            children: None,
        }
    }

    fn new<S: SubsetMaskStage>(max_depth: u8) -> Self {
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
        let mut ret = TrieNode::with_single_entry(init_mask.subset_trie_key(), total_bits, 0);
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
        let alloc = super::thread_local_bump_allocator();
        for (i, (&k, &v)) in new_hashmap.iter().enumerate() {
            ret.insert(alloc, k, total_bits, v);
            if i % 1_000_000 == 0 && i > 0 {
                println!("  done {}/{}M", i / 1_000_000, entry_count / 1_000_000);
            }
        }
        ret
    }

    fn serialize(&self) -> Vec<u8> {
        let mut buf = vec![];
        self.ser_to_buf(&mut BitWriteStream::new(&mut buf, LittleEndian))
            .unwrap();
        buf
    }

    fn deserialize(buf: &[u8]) -> bitbuffer::Result<Self> {
        Self::deser_from_buf(
            super::thread_local_bump_allocator(),
            &mut BitReadStream::new(BitReadBuffer::new(&buf, LittleEndian)),
        )
    }

    fn ser_to_buf(&self, buf: &mut BitWriteStream<'_, LittleEndian>) -> bitbuffer::Result<()> {
        let Self {
            mask_len,
            mask,
            lower_bound,
            children,
        } = self;
        buf.write_int(*mask_len, 8)?;
        buf.write_int(*mask, *mask_len as usize)?;
        buf.write_bool(children.is_some())?;
        if let Some(children) = children {
            children[0].ser_to_buf(buf)?;
            children[1].ser_to_buf(buf)?;
            assert_eq!(
                *lower_bound,
                children[0].lower_bound.min(children[1].lower_bound)
            );
        } else {
            buf.write_int(*lower_bound, DEPTH_BITS)?;
        }
        Ok(())
    }

    fn deser_from_buf(
        alloc: &'static bumpalo::Bump,
        buf: &mut BitReadStream<'_, LittleEndian>,
    ) -> bitbuffer::Result<Self> {
        let mask_len = buf.read_int::<u8>(8)?;
        let mask = buf.read_int::<u128>(mask_len as usize)?;
        let children = if buf.read_bool()? {
            Some(alloc.alloc(TrieChildren([
                Self::deser_from_buf(alloc, buf)?,
                Self::deser_from_buf(alloc, buf)?,
            ])))
        } else {
            None
        };
        let lower_bound = match &children {
            Some(children) => std::cmp::min(children[0].lower_bound, children[1].lower_bound),
            None => buf.read_int(DEPTH_BITS)?,
        };
        Ok(Self {
            mask_len,
            mask,
            lower_bound,
            children,
        })
    }
}

/// Wrapper around `[TrieNode; 2]` for cache alignment.
#[derive(Debug, PartialEq, Eq)]
#[repr(align(64))]
struct TrieChildren([TrieNode; 2]);

impl Deref for TrieChildren {
    type Target = [TrieNode; 2];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TrieChildren {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::stages::Stage1;

    #[test]
    fn test_pruning_trie_ser_deser() {
        let pruning_trie = TrieNode::new::<Stage1>(2);
        let serialized = pruning_trie.serialize();
        let deserialized = TrieNode::deserialize(&serialized).unwrap();
        assert_eq!(deserialized, pruning_trie);
    }

    #[test]
    fn test_pruning_trie_determinism() {
        let trie1 = TrieNode::new::<Stage1>(2);
        let trie2 = TrieNode::new::<Stage1>(2);
        assert_eq!(trie1, trie2);
    }
}
