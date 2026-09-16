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

/// Returns an iterator over permutations of a list, each with whether it is
/// odd.
pub fn permutations_with_parity<I>(iter: I) -> impl Iterator<Item = (Vec<I::Item>, bool)>
where
    I: ExactSizeIterator,
    I::Item: Clone,
{
    let len = iter.len();
    iter.permutations(len)
        .enumerate()
        .map(|(i, p)| (p, is_permutation_odd(i)))
}

/// Returns the parity of the permutation with number `n`.
pub fn is_permutation_odd(mut n: usize) -> bool {
    let mut res = false;
    let mut i = 2;
    while n > 0 {
        res ^= !(n % i).is_multiple_of(2);
        n /= i;
        i += 1;
    }
    res
}
