use std::ops::Range;

use crate::{address::DmxAddress, constants::DMX_CHANNELS};

/// A conceptually contiguous range of DMX channels across consecutive
/// universes, potentially with padding inbetween to improve alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DmxAllocation {
    start_address: DmxAddress,
    count: usize,
    grouping: usize,
}

impl DmxAllocation {
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
                    let preceding_mid = (address.universe() - range.start.universe()) * self.mid_universe_channel_count();
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
    
    pub fn address_range(&self) -> Range<DmxAddress> {
        self.start_address()..self.end_address()
    }

    pub fn start_address(&self) -> DmxAddress {
        self.start_address
    }

    pub fn end_address(&self) -> DmxAddress {
        let start_channels = self.start_universe_channel_count();
        let mid_channels = self.mid_universe_channel_count();
        if self.count <= start_channels {
            self.start_address + self.count
        } else {
            let universes = (self.count - start_channels).div_ceil(mid_channels);
            let preceding = start_channels + universes * mid_channels;
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
