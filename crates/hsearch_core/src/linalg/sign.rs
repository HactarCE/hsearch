#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Sign {
    Pos = 1,
    Neg = -1,
}

impl Sign {
    /// Returns the sign of a nonzero number.
    ///
    /// # Panics
    ///
    /// Panics if `i == 0`.
    #[track_caller]
    pub const fn from_i8(i: i8) -> Self {
        if i > 0 {
            Sign::Pos
        } else if i < 0 {
            Sign::Neg
        } else {
            panic!("cannot take sign of zero")
        }
    }

    pub const fn from_lsb(b: u8) -> Self {
        if b & 1 == 0 { Self::Pos } else { Self::Neg }
    }

    #[inline(never)]
    #[unsafe(no_mangle)]
    pub const fn to_lsb(self) -> u8 {
        // 0 for Pos, 1 for Neg
        self as u8 >> 7
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_to_lsb() {
        assert_eq!(Sign::Pos.to_lsb(), 0);
        assert_eq!(Sign::Neg.to_lsb(), 1);
    }
}
