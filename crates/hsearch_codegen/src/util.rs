use hsearch_core::{PieceType, Vec4};

pub fn ridges() -> impl Iterator<Item = Vec4> {
    PieceType::Ridge.iter()
}

pub fn edges() -> impl Iterator<Item = Vec4> {
    PieceType::Edge.iter()
}

pub fn corners() -> impl Iterator<Item = Vec4> {
    PieceType::Corner.iter()
}

pub fn preserved_bits(
    int_width: usize,
    bit_offset: usize,
    bits_per_element: usize,
    piece_count: usize,
) -> u128 {
    !1_u128
        .unbounded_shl((bit_offset + bits_per_element * piece_count) as u32)
        .wrapping_sub(1_u128.strict_shl(bit_offset as u32))
        & 1_u128.unbounded_shl(int_width as u32).wrapping_sub(1)
}
