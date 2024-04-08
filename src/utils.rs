use std::{cmp::{max, min}, ops::Range};

pub trait RangeExt: Sized {
    fn intersect(self, rhs: Self) -> Option<Self>;
}

impl<Idx> RangeExt for Range<Idx> where Idx: Ord {
    fn intersect(self, rhs: Self) -> Option<Self> {
        let start = max(self.start, rhs.start);
        let end = min(self.end, rhs.end);
        if start <= end {
            Some(start..end)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::RangeExt;

    #[test]
    fn simple() {
        assert_eq!((1..3).intersect(1..3), Some(1..3));
        assert_eq!((1..3).intersect(2..3), Some(2..3));
        assert_eq!((1..3).intersect(2..4), Some(2..3));
        assert_eq!((1..3).intersect(1..1), Some(1..1));
        assert_eq!((1..3).intersect(3..3), Some(3..3));
        assert_eq!((1..3).intersect(0..0), None);
        assert_eq!((1..1).intersect(0..0), None);
    }
}
