/// Number of pieces data is split into for coding.
///
/// Guaranteed to be non-zero by construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PieceCount(usize);

impl PieceCount {
    /// Creates a new `PieceCount` from the given value.
    ///
    /// Returns `None` if `value` is zero.
    pub const fn new(value: usize) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self(value))
        }
    }

    /// Returns the underlying count as a `usize`.
    pub const fn value(self) -> usize {
        self.0
    }
}

/// Byte length of each data piece after padding.
///
/// Guaranteed to be non-zero by construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PieceByteLen(usize);

impl PieceByteLen {
    /// Creates a new `PieceByteLen` from the given value.
    ///
    /// Returns `None` if `value` is zero.
    pub const fn new(value: usize) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self(value))
        }
    }

    /// Returns the underlying length as a `usize`.
    pub const fn value(self) -> usize {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn piece_count_rejects_zero() {
        assert_eq!(PieceCount::new(0), None);
    }

    #[test]
    fn piece_count_accepts_non_zero() {
        let pc = PieceCount::new(32).unwrap();
        assert_eq!(pc.value(), 32);
    }

    #[test]
    fn piece_byte_len_rejects_zero() {
        assert_eq!(PieceByteLen::new(0), None);
    }

    #[test]
    fn piece_byte_len_accepts_non_zero() {
        let pbl = PieceByteLen::new(1024).unwrap();
        assert_eq!(pbl.value(), 1024);
    }
}
