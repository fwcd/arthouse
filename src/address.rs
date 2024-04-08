use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DmxAddress(u32);

impl DmxAddress {
    pub const ZERO: Self = Self(0);

    #[inline]
    pub fn new(universe: u16, channel: u8) -> Self {
        Self(((universe as u32) << 8) | (channel as u32))
    }

    #[inline]
    pub fn universe(self) -> u16 {
        (self.0 >> 8) as u16
    }

    #[inline]
    pub fn channel(self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    #[inline]
    pub fn value(self) -> u32 {
        self.0
    }
}

impl Default for DmxAddress {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<(u16, u8)> for DmxAddress {
    fn from((universe, channel): (u16, u8)) -> Self {
        Self::new(universe, channel)
    }
}

impl From<DmxAddress> for (u16, u8) {
    fn from(address: DmxAddress) -> Self {
        (address.universe(), address.channel())
    }
}

impl Add<u32> for DmxAddress {
    type Output = Self;

    fn add(self, rhs: u32) -> Self {
        Self(self.0 + rhs)
    }
}

impl Add<DmxAddress> for DmxAddress {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Sub<u32> for DmxAddress {
    type Output = Self;

    fn sub(self, rhs: u32) -> Self {
        Self(self.0 - rhs)
    }
}

impl Sub<DmxAddress> for DmxAddress {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl AddAssign<u32> for DmxAddress {
    fn add_assign(&mut self, rhs: u32) {
        self.0 += rhs;
    }
}

impl AddAssign<DmxAddress> for DmxAddress {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign<u32> for DmxAddress {
    fn sub_assign(&mut self, rhs: u32) {
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
        assert_eq!(DmxAddress::new(2, 3).0, 0x0203);
        assert_eq!(DmxAddress::new(2, 3).universe(), 2);
        assert_eq!(DmxAddress::new(2, 3).channel(), 3);
        assert_eq!(<(u16, u8)>::from(DmxAddress::ZERO), (0, 0));
        assert_eq!(<(u16, u8)>::from(DmxAddress::new(16, 254) + 3), (17, 1));
    }
}
