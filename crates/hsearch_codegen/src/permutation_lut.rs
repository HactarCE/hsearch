use std::collections::{BTreeMap, HashMap};

use hsearch_core::{Twist, TwistSet, Vec4};
use itertools::Itertools;

const INDENT: &str = "                ";

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
            Some(if t.affects(p) { t.rot() * p } else { p })
        })
    }

    pub fn with_action(
        pieces: impl IntoIterator<Item = Vec4>,
        mut act: impl FnMut(Twist, Vec4) -> Option<Vec4>,
    ) -> Self {
        let pieces = pieces.into_iter().collect_vec();
        let point_to_index: HashMap<Vec4, usize> =
            pieces.iter().enumerate().map(|(i, &p)| (p, i)).collect();
        Self {
            piece_count: pieces.len(),
            table: Twist::iter()
                .map(|t| {
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

        let rows = self
            .table
            .iter()
            .enumerate()
            .filter_map(|(i, opt_row)| {
                let row = opt_row.as_ref()?;
                let mut delta_masks = BTreeMap::<usize, u128>::new();
                delta_masks.insert(0, preserved_bits);
                for (src, &dst) in row.iter().enumerate() {
                    let mask = element_mask(src);
                    let src = src * bits_per_element + bit_offset;
                    let dst = dst * bits_per_element + bit_offset;
                    let delta = dst.wrapping_sub(src) % int_width;
                    *delta_masks.entry(delta).or_default() |= mask;
                }
                let shift_masks = delta_masks
                    .iter()
                    .map(|(delta, mask)| format!("(&0x{mask:X}<<{delta})"))
                    .join("|");
                Some(format!("{INDENT}    {i} => [{shift_masks}],"))
            })
            .join("\n");

        format!(
            "apply_permutation_lut!(u{int_width}, {state_var}, {twist_var}, [\n{rows}\n{INDENT}])"
        )
    }

    /// Returns the twists supported by the permutation.
    pub fn allowed_twists(&self) -> TwistSet {
        TwistSet::new(|t| self.table[t.index() as usize].is_some())
    }
}
