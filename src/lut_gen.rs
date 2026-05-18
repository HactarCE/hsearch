//! Code generator for lookup tables.
#![cfg_attr(not(test), allow(dead_code))]

use std::collections::BTreeMap;
use std::collections::HashMap;

use itertools::Itertools;

use crate::HYPERCUBE_TWISTS;
use crate::prelude::*;

/// Lookup table for permuting pieces.
pub struct PermutationLut {
    piece_count: usize,
    /// For each twist, for each point: the new point.
    table: Vec<Option<Vec<usize>>>,
}

impl PermutationLut {
    /// Generates a lookup table that permutes pieces according to each twist.
    pub fn new(pieces: impl IntoIterator<Item = Vec4>) -> Self {
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
                        .map(|&p| {
                            point_to_index
                                .get(&if t.affects(p) { t.rot * p } else { p })
                                .copied()
                        })
                        .collect()
                })
                .collect(),
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

        let element_mask =
            |p| ((1_u64 << bits_per_element) - 1) << (p * bits_per_element + bit_offset);

        let mut s = String::new();
        s += &format!("apply_permutation_lut!(u{int_width}, {state_var}, {twist_var}, [\n");
        for (i, opt_row) in self.table.iter().enumerate() {
            if let Some(row) = opt_row {
                let mut delta_masks = BTreeMap::<usize, u64>::new();
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

/// Lookup table for updating piece orientations.
pub struct OrientationLut {
    /// For each twist, for each piece location, for each orientation: the new
    /// orientation.
    table: Vec<Vec<Vec<u8>>>,
}

impl OrientationLut {
    /// Generates a lookup table that updates the orientation for each piece
    /// according to a twist.
    pub fn new(
        pieces: impl IntoIterator<Item = Vec4>,
        orientation_count: u8,
        act: impl Fn(Mat4, Vec4, u8) -> u8,
    ) -> Self {
        let pieces = pieces.into_iter().collect_vec();
        Self {
            table: HYPERCUBE_TWISTS
                .iter()
                .map(|&t| {
                    pieces
                        .iter()
                        .map(|&p| {
                            (0..orientation_count)
                                .map(|o| if t.affects(p) { act(t.rot, p, o) } else { o })
                                .collect()
                        })
                        .collect()
                })
                .collect(),
        }
    }

    /// Returns Rust source code for updating piece orientations for a twist.
    ///
    /// Orientations must be updated before pieces are permuted.
    pub fn to_rust_code(
        &self,
        int_width: usize,
        bit_offset: usize,
        bits_per_element: usize,
        state_var: &str,
    ) -> String {
        let twist_var = "twist";

        let point_count = self.table[0].len();
        assert!(point_count * bits_per_element + bit_offset <= int_width);

        let orientation_count = self.table[0][0].len();
        assert!(orientation_count <= 1 << bits_per_element);

        if orientation_count > 4 {
            panic!("LUTs are only supported for up to 4 orientations");
        }

        let mut s = String::new();
        s += &format!("crate::lut::update_orientations_u{int_width}(\n");
        s += &format!("    {state_var},\n");
        s += "    [\n";

        for row in &self.table {
            // Consider the case of 4 orientations, each represented by 2 bits.
            // Each bit in the output is a function of exactly two bits of the
            // input (itself and its neighbor), which we'll call `a` (itself)
            // and `b` (its neighbor). In particular, that function returns `0`
            // for half of input cases and `1` for the other half. There are
            // only six possible functions:
            //
            // - `a`
            // - `b`
            // - `!a`
            // - `!b`
            // - `a ^ b`
            // - `!a ^ b`
            //
            // We can compute any of these functions using the formula `m1 ^ (ma
            // & a) ^ (mb & b)` for some masks `m1`, `ma`, and `mb`. Our goal is
            // to compute those masks.
            let mut m1 = 0;
            let mut ma = 0;
            let mut mb = 0;
            for (j, orientation_map) in row.iter().enumerate() {
                let l_for_0 = orientation_map[0] & 1 != 0;
                let l_depends_on_l = orientation_map[0] & 1 != orientation_map[1] & 1;
                let l_depends_on_h = orientation_map[0] & 1 != orientation_map[2] & 1;
                let h_for_0 = orientation_map[0] & 2 != 0;
                let h_depends_on_l = orientation_map[0] & 2 != orientation_map[1] & 2;
                let h_depends_on_h = orientation_map[0] & 2 != orientation_map[2] & 2;
                let l_offset = j * 2;
                let h_offset = l_offset + 1;
                m1 |= (l_for_0 as u64) << l_offset;
                ma |= (l_depends_on_l as u64) << l_offset;
                mb |= (l_depends_on_h as u64) << l_offset;
                m1 |= (h_for_0 as u64) << h_offset;
                ma |= (h_depends_on_h as u64) << h_offset;
                mb |= (h_depends_on_l as u64) << h_offset;
            }
            s += &format!("        [0x{m1:X}, 0x{ma:X}, 0x{mb:X}],\n");
        }

        s += &format!("    ][{twist_var}.to_index()],\n");
        s += ")\n";
        s
    }
}
