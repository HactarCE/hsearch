use std::{fmt, range::RangeInclusive};

use itertools::Itertools;

use crate::linalg::*;

/// Bitmask indicating a hyperrectangular region of pieces on the puzzle.
///
/// This is represented using a total of 12 bits, 3 for each axis, representing
/// the negative facet layer, middle slice layer, and positive facet layer in
/// that order.
#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Block(u16);

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Block")
            .field("x", &self.axis_bits(X))
            .field("y", &self.axis_bits(Y))
            .field("z", &self.axis_bits(Z))
            .field("w", &self.axis_bits(W))
            .field("extra_bits", &(self.0 >> 12))
            .finish()
    }
}

impl Block {
    pub const EMPTY: Self = Self(0);

    pub fn print_dims(self) {
        println!(
            "{}",
            Axis::ALL
                .map(|ax| self.axis_bits(ax).count_ones().to_string())
                .into_iter()
                .join("x")
        )
    }

    pub fn piece_count(self) -> u32 {
        Axis::ALL
            .map(|ax| self.axis_bits(ax).count_ones())
            .iter()
            .product()
    }

    /// Returns whether the block is equal to [`Block::EMPTY`].
    pub const fn is_empty(self) -> bool {
        self.0 == Self::EMPTY.0
    }

    /// Constructs a block from a range along each axis.
    ///
    /// Coordinates outside the puzzle (±1) are safely ignored.
    pub const fn new(axis_ranges: [RangeInclusive<i8>; 4]) -> Self {
        let mut ret: u16 = 0;
        let mut i: usize = 0;
        while i < 12 {
            let axis_index = i / 3;
            let coordinate = (i % 3) as i8 - 1;
            let axis_range = &axis_ranges[axis_index];
            let has_layer = axis_range.start <= coordinate && coordinate <= axis_range.last;
            ret |= (has_layer as u16) << i;
            i += 1;
        }
        Self(ret)
    }

    /// Returns whether the block intersects a middle slice layer.
    #[inline(always)]
    pub const fn intersects_slice_layer(self, axis: Axis) -> bool {
        (self.0 >> Self::slice_index(axis)) & 1 != 0
    }

    /// Returns whether the block intersects a facet layer.
    #[inline(always)]
    pub const fn intersects_facet_layer(self, facet: Facet) -> bool {
        (self.0 >> Self::facet_index(facet)) & 1 != 0
    }

    /// Returns whether a piece is contained in the block.
    pub fn contains(self, v: Vec4) -> bool {
        Axis::ALL
            .into_iter()
            .all(|ax| (self.0 >> Self::axis_layer_index(ax, v[ax])) & 1 != 0)
    }

    fn axis_layer_index(axis: Axis, layer: i8) -> u8 {
        (axis as i8 * 3 + 1 + layer.clamp(-1, 1)) as u8
    }

    /// Returns the bit index for a facet layer.
    const fn facet_index(facet: Facet) -> u8 {
        (facet.axis() as i8 * 3 + 1 + facet.sign() as i8) as u8
    }

    /// Returns the bit index for a middle slice layer.
    const fn slice_index(axis: Axis) -> u8 {
        axis as u8 * 3 + 1
    }

    /// Returns the 3 bits representing the block's position along one axis.
    const fn axis_bits(self, axis: Axis) -> u16 {
        (self.0 >> (axis as u8 * 3)) & 0b111
    }
}

impl TransformByMat4 for Block {
    fn transform_by(&self, m: Mat4) -> Self {
        let [x, y, z, w] = Axis::ALL.map(|ax| {
            let old_axis_bits = self.axis_bits(ax) & 0b111;
            let new_axis_bits = match m.col(ax).sign() {
                Sign::Pos => old_axis_bits,
                Sign::Neg => old_axis_bits.reverse_bits() >> 13, // reverse 3 lowest bits
            };
            new_axis_bits << (m.col(ax).axis() as u8 * 3)
        });
        Self(x | y | z | w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_construction() {
        for axis_ranges in all_possible_block_constructions() {
            let block = Block::new(axis_ranges);
            for axis in Axis::ALL {
                let axis_range = axis_ranges[axis as usize];
                assert_eq!(axis_range.contains(&0), block.intersects_slice_layer(axis));
                assert_eq!(
                    axis_range.contains(&1),
                    block.intersects_facet_layer(Facet::pos(axis)),
                );
                assert_eq!(
                    axis_range.contains(&-1),
                    block.intersects_facet_layer(Facet::neg(axis)),
                );
            }
        }
    }

    fn all_possible_block_constructions() -> Vec<[RangeInclusive<i8>; 4]> {
        let possible_ranges: [RangeInclusive<i8>; _] = [
            // length 1
            -1..=-1,
            0..=0,
            1..=1,
            // length 2
            -1..=0,
            0..=1,
            // length 3
            -1..=1,
        ]
        .map(RangeInclusive::from);
        itertools::iproduct!(
            possible_ranges,
            possible_ranges,
            possible_ranges,
            possible_ranges
        )
        .map(|(x, y, z, w)| [x, y, z, w])
        .collect()
    }

    #[test]
    fn test_transform_solved_block() {
        for rot in crate::Group::hypercube_rotations().elems() {
            for axis_ranges in all_possible_block_constructions() {
                let old_block = Block::new(axis_ranges);
                let new_block = old_block.transform_by(rot);
                for axis in Axis::ALL {
                    assert_eq!(
                        old_block.intersects_slice_layer(axis),
                        new_block.intersects_slice_layer(axis.transform_by(rot))
                    );
                }
                for facet in Facet::ALL {
                    assert_eq!(
                        old_block.intersects_facet_layer(facet),
                        new_block.intersects_facet_layer(facet.transform_by(rot))
                    );
                }
            }
        }
    }

    #[test]
    fn test_solved_block_contains_point() {
        for axis_ranges in all_possible_block_constructions() {
            let block = Block::new(axis_ranges);
            for v in Vec4::region(Vec4([-1; 4]), Vec4([1; 4])) {
                assert_eq!(
                    axis_ranges[0].contains(&v[X])
                        && axis_ranges[1].contains(&v[Y])
                        && axis_ranges[2].contains(&v[Z])
                        && axis_ranges[3].contains(&v[W]),
                    block.contains(v),
                );
            }
        }
    }
}
