use std::ops::Range;

use crate::{address::DmxAddress, constants::DMX_CHANNELS};

/// A range of DMX channels across consecutive universes, potentially with
/// padding to avoid splitting groups of channels across universes. For a group
/// size of 1, the allocation of channels is fully contiguous.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DmxAllocation {
    /// The first DMX channel to use.
    start_address: DmxAddress,
    /// The total number of DMX channels.
    count: usize,
    /// The number of DMX channels per group (which cannot be split across universes).
    grouping: usize,
}

impl DmxAllocation {
    /// Creates a new DMX allocation from the given start address, channel count and group size.
    pub fn new(start_address: DmxAddress, count: usize, grouping: usize) -> Self {
        Self { start_address, count, grouping }
    }

    /// Fetches the logical index of the channel for the given DMX address, if it exists.
    pub fn index_of(&self, address: DmxAddress) -> Option<usize> {
        let range = self.address_range();
        if range.contains(&address) {
            let (preceding, index, remaining) =
                if address.universe() == range.start.universe() {
                    let preceding = 0;
                    let index = address.channel() - range.start.channel();
                    let remaining = self.start_universe_channel_count();
                    (preceding, index, remaining)
                } else {
                    let preceding_start = self.start_universe_channel_count();
                    let preceding_mid = (address.universe() - range.start.universe() - 1) * self.mid_universe_channel_count();
                    let preceding = preceding_start + preceding_mid;
                    let index = address.channel();
                    let remaining = if address.universe() == range.end.universe() {
                        self.end_universe_channel_count()
                    } else {
                        self.mid_universe_channel_count()
                    };
                    (preceding, index, remaining)
                };

            Some(preceding + index).filter(|_| index < remaining)
        } else {
            None
        }
    }

    /// Fetches the allocated (contiguous) range of DMX channels in the given
    /// universe. May be empty, e.g. if the universe is outside the allocation.
    pub fn address_range_in(&self, universe: usize) -> Range<DmxAddress> {
        let start = if universe == self.start_address.universe() {
            self.start_address
        } else {
            DmxAddress::new(universe, 0)
        };
        start..(start + self.universe_channel_count(universe))
    }

    fn universe_channel_count(&self, universe: usize) -> usize {
        let range = self.address_range();
        if universe < range.start.universe() || universe > range.end.universe() {
            0
        } else if universe == range.start.universe() {
            self.start_universe_channel_count()
        } else if universe == range.end.universe() {
            self.end_universe_channel_count()
        } else {
            self.mid_universe_channel_count()
        }
    }

    fn end_universe_channel_count(&self) -> usize {
        self.end_address().channel()
    }
    
    /// Fetches the total range of allocated DMX channels.
    pub fn address_range(&self) -> Range<DmxAddress> {
        self.start_address()..self.end_address()
    }

    /// Fetches the first allocated DMX channel (inclusive start).
    pub fn start_address(&self) -> DmxAddress {
        self.start_address
    }

    /// Fetches the DMX channel past the last allocated channel (exclusive end).
    pub fn end_address(&self) -> DmxAddress {
        let start_channels = self.start_universe_channel_count();
        let mid_channels = self.mid_universe_channel_count();
        if self.count <= start_channels {
            self.start_address + self.count
        } else {
            let universes = (self.count - start_channels).div_ceil(mid_channels);
            let preceding = start_channels + (universes - 1) * mid_channels;
            DmxAddress::new(self.start_address.universe() + universes, self.count - preceding)
        }
    }

    fn start_universe_channel_count(&self) -> usize {
        let remaining = DMX_CHANNELS - self.start_address.channel();
        remaining - remaining % self.grouping
    }

    fn mid_universe_channel_count(&self) -> usize {
        DMX_CHANNELS - DMX_CHANNELS % self.grouping
    }
}

#[cfg(test)]
mod tests {
    use lighthouse_client::protocol::{LIGHTHOUSE_BYTES, LIGHTHOUSE_COLS};

    use crate::{address::DmxAddress, allocation::DmxAllocation, constants::DMX_CHANNELS};

    #[test]
    fn full_universe() {
        // A simple contiguous allocation of 512 channels in a single universe
        let allocation = DmxAllocation::new(DmxAddress::new(1, 0), DMX_CHANNELS, 1);

        assert_eq!(allocation.start_universe_channel_count(), 512);
        assert_eq!(allocation.start_address(), DmxAddress::new(1, 0));
        assert_eq!(allocation.end_address(), DmxAddress::new(2, 0));

        assert!(allocation.index_of(DmxAddress::new(0, 0)).is_none());
        assert!(allocation.index_of(DmxAddress::new(0, 511)).is_none());
        assert_eq!(allocation.index_of(DmxAddress::new(1, 0)), Some(0));
        assert_eq!(allocation.index_of(DmxAddress::new(1, 1)), Some(1));
        assert_eq!(allocation.index_of(DmxAddress::new(1, 511)), Some(511));
        assert!(allocation.index_of(DmxAddress::new(2, 0)).is_none());
    }

    #[test]
    fn groups_of_three() {
        // 512 channels, starting in universe 1, using groups of 3, like e.g. for RGB.
        // This introduces some padding at the end of universe 1 and moves the
        // last group to universe 2 to ensure that no group is split across universes.
        let allocation = DmxAllocation::new(DmxAddress::new(1, 0), DMX_CHANNELS, 3);

        assert_eq!(allocation.start_universe_channel_count(), 510);
        assert_eq!(allocation.end_universe_channel_count(), 2); // Last group has only 2 channels (512 % 3 == 2)
        assert_eq!(allocation.start_address(), DmxAddress::new(1, 0));
        assert_eq!(allocation.end_address(), DmxAddress::new(2, 2));

        assert!(allocation.index_of(DmxAddress::new(0, 0)).is_none());
        assert!(allocation.index_of(DmxAddress::new(0, 511)).is_none());
        assert_eq!(allocation.index_of(DmxAddress::new(1, 0)), Some(0));
        assert_eq!(allocation.index_of(DmxAddress::new(1, 1)), Some(1));
        assert_eq!(allocation.index_of(DmxAddress::new(1, 509)), Some(509));
        assert!(allocation.index_of(DmxAddress::new(1, 510)).is_none());
        assert_eq!(allocation.index_of(DmxAddress::new(2, 0)), Some(510));
        assert_eq!(allocation.index_of(DmxAddress::new(2, 1)), Some(511));
        assert!(allocation.index_of(DmxAddress::new(2, 2)).is_none());
    }

    #[test]
    fn lighthouse() {
        // 14 * 28 * 3 = 1176 channels, starting in universe 1, using groups of 28 * 3 = 84
        // to ensure that full rows of pixels are grouped together.
        let allocation = DmxAllocation::new(DmxAddress::new(1, 0), LIGHTHOUSE_BYTES, LIGHTHOUSE_COLS * 3);

        assert_eq!(allocation.start_universe_channel_count(), 504); // DMX_CHANNELS = 512 rounded down to multiple of 84
        assert_eq!(allocation.mid_universe_channel_count(), 504); 
        assert_eq!(allocation.end_universe_channel_count(), 168); // 1176 - 2 * 504
    }

    #[test]
    fn misaligned_lighthouse() {
        // 14 * 28 * 3 = 1176 channels, starting in universe 1, using groups of 28 * 3 = 84
        // to ensure that full rows of pixels are grouped together.
        let allocation = DmxAllocation::new(DmxAddress::new(1, 200), LIGHTHOUSE_BYTES, LIGHTHOUSE_COLS * 3);

        assert_eq!(allocation.start_universe_channel_count(), 252); // 512 - 200 rounded down to multiple of 84
        assert_eq!(allocation.mid_universe_channel_count(), 504); // 512 rounded down to multiple of 84
        assert_eq!(allocation.end_universe_channel_count(), 420); // 1176 - 252 - 504
    }
}
