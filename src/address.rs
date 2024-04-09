use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DmxAddress(usize);

impl DmxAddress {
    pub const ZERO: Self = Self(0);

    #[inline]
    pub fn new(universe: usize, channel: usize) -> Self {
        Self((universe << 9) | channel)
    }

    #[inline]
    pub fn universe(self) -> usize {
        self.0 >> 9
    }

    #[inline]
    pub fn channel(self) -> usize {
        self.0 & 0x1FF
    }

    #[inline]
    pub fn value(self) -> usize {
        self.0
    }
}

impl Default for DmxAddress {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<usize> for DmxAddress {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<(usize, usize)> for DmxAddress {
    fn from((universe, channel): (usize, usize)) -> Self {
        Self::new(universe, channel)
    }
}

impl From<DmxAddress> for (usize, usize) {
    fn from(address: DmxAddress) -> Self {
        (address.universe(), address.channel())
    }
}

impl Add<usize> for DmxAddress {
    type Output = Self;

    fn add(self, rhs: usize) -> Self {
        Self(self.0 + rhs)
    }
}

impl Add<DmxAddress> for DmxAddress {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Sub<usize> for DmxAddress {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self {
        Self(self.0 - rhs)
    }
}

impl Sub<DmxAddress> for DmxAddress {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl AddAssign<usize> for DmxAddress {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl AddAssign<DmxAddress> for DmxAddress {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign<usize> for DmxAddress {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs;
    }
}

impl SubAssign<DmxAddress> for DmxAddress {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

#[cfg(test)]
mod tests {
    use crate::address::DmxAddress;

    #[test]
    fn addresses() {
        // Notice the extra 9th bit, here -------v
        assert_eq!(DmxAddress::new(2, 3).0, 0b10_000000011);
        assert_eq!(DmxAddress::new(2, 3).universe(), 2);
        assert_eq!(DmxAddress::new(2, 3).channel(), 3);
        assert_eq!(<(usize, usize)>::from(DmxAddress::ZERO), (0, 0));
        assert_eq!(<(usize, usize)>::from(DmxAddress::new(16, 510) + 3), (17, 1));
    }

    #[test]
    fn carry() {
        assert_eq!(DmxAddress::new(0, 511) + 1, DmxAddress::new(1, 0));
        assert_eq!(DmxAddress::new(0, 512), DmxAddress::new(1, 0));
        assert_eq!(DmxAddress::new(1, 2) + 1025, DmxAddress::new(3, 3));
    }
}
