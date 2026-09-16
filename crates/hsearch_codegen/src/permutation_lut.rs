use std::collections::BTreeMap;
use std::collections::HashMap;

use itertools::Itertools;

use hsearch_core::{HYPERCUBE_TWISTS, Twist, TwistData, Vec4};

/// Lookup table for permuting pieces.
pub struct PermutationLut {
    piece_count: usize,
    /// For each twist, for each point: the new point.
    table: Vec<Option<Vec<usize>>>,
}

impl PermutationLut {
    /// Generates a lookup table that permutes pieces according to each twist.
    pub fn new(pieces: impl IntoIterator<Item = Vec4>) -> Self {
        Self::with_action(pieces, |t, p| {
            Some(if t.affects(p) { t.rot * p } else { p })
        })
    }

    pub fn with_action(
        pieces: impl IntoIterator<Item = Vec4>,
        mut act: impl FnMut(TwistData, Vec4) -> Option<Vec4>,
    ) -> Self {
        let pieces = pieces.into_iter().collect_vec();
        let point_to_index: HashMap<Vec4, usize> =
            pieces.iter().enumerate().map(|(i, &p)| (p, i)).collect();
        Self {
            piece_count: pieces.len(),
            table: HYPERCUBE_TWISTS
                .iter()
                .map(|&t| {
                    pieces
                        .iter()
                        .map(|&p| point_to_index.get(&act(t, p)?).copied())
                        .collect::<Option<Vec<_>>>()
                })
                .collect::<Vec<_>>(),
        }
    }

    /// Returns Rust source code for applying the permutation.
    pub fn to_rust_code(
        &self,
        int_width: usize,
        bit_offset: usize,
        bits_per_element: usize,
        state_var: &str,
    ) -> String {
        let twist_var = "twist";

        assert!(
            self.piece_count * bits_per_element + bit_offset <= int_width,
            "integer is not wide enough",
        );

        let preserved_bits =
            crate::preserved_bits(int_width, bit_offset, bits_per_element, self.piece_count);

        let element_mask =
            |p| ((1_u128 << bits_per_element) - 1) << (p * bits_per_element + bit_offset);

        let mut s = String::new();
        s += &format!("apply_permutation_lut!(u{int_width}, {state_var}, {twist_var}, [\n");
        for (i, opt_row) in self.table.iter().enumerate() {
            let Some(row) = opt_row else { continue };
            let mut delta_masks = BTreeMap::<usize, u128>::new();
            delta_masks.insert(0, preserved_bits);
            for (src, &dst) in row.iter().enumerate() {
                let mask = element_mask(src);
                let src = src * bits_per_element + bit_offset;
                let dst = dst * bits_per_element + bit_offset;
                let delta = dst.wrapping_sub(src) % int_width;
                *delta_masks.entry(delta).or_default() |= mask;
            }
            s += &format!("    {i} => [");
            s += &delta_masks
                .iter()
                .map(|(delta, mask)| format!("(&0x{mask:X}<<{delta})"))
                .join("|");
            s += "],\n";
        }
        s += "])";
        s
    }

    /// Returns the twists supported by the permutation.
    pub fn allowed_twists(&self) -> Vec<Twist> {
        Twist::iter()
            .filter(|t| self.table[t.to_index()].is_some())
            .collect()
    }
}
