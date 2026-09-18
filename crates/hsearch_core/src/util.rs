use itertools::Itertools;

/// Collects booleans into a bitmask.
///
/// # Panics
///
/// Panics in debug mode if there are too many bits.
pub fn collect_bits<B: num_traits::PrimInt>(iter: impl IntoIterator<Item = bool>) -> B {
    iter.into_iter()
        .positions(|b| b)
        .map(|i| B::one() << i)
        .fold(B::zero(), |a, b| a | b)
}
