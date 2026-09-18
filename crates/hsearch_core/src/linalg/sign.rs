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
}
