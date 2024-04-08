use std::ops::Range;

use anyhow::Result;
use artnet_protocol::ArtCommand;
use lighthouse_client::{protocol::LIGHTHOUSE_BYTES, Lighthouse, TokioWebSocket};
use tokio::net::UdpSocket;
use tracing::{info, info_span, warn, Instrument};

use crate::{address::DmxAddress, utils::RangeExt};

pub struct ArtNetAdapter {
    lh: Lighthouse<TokioWebSocket>,
    socket: UdpSocket,
    start_address: DmxAddress,
    frame: [u8; LIGHTHOUSE_BYTES],
}

impl ArtNetAdapter {
    pub fn new(lh: Lighthouse<TokioWebSocket>, socket: UdpSocket, start_address: DmxAddress) -> Self {
        Self { lh, socket, start_address, frame: [0u8; LIGHTHOUSE_BYTES] }
    }

    pub async fn run(mut self) -> Result<()> {
        info!("Listening for Art-Net packets on {} (UDP)", self.socket.local_addr()?);

        loop {
            let mut buffer = [0u8; 1024];
            let (length, addr) = self.socket.recv_from(&mut buffer).await?;
            async {
                let result = self.handle_raw(&buffer[..length]).await;
                if let Err(e) = result {
                    warn!(error = %e, "Error while parsing packet");
                }
            }
            .instrument(info_span!("Client", %addr))
            .await;
        }
    }

    async fn handle_raw(&mut self, raw: &[u8]) -> Result<()> {
        let command = ArtCommand::from_buffer(raw)?;
        self.handle_command(command).await
    }

    async fn handle_command(&mut self, command: ArtCommand) -> Result<()> {
        match command {
            ArtCommand::Output(output) => {
                let port_address = output.port_address.into();
                info! {
                    version = ?output.version,
                    sequence = output.sequence,
                    port_address = port_address,
                    length = *output.length,
                    "Handling output"
                };
                let packet_range = DmxAddress::new(port_address, 0)..DmxAddress::new(port_address + 1, 0);
                let address_range = self.address_range();
                if let Some(relevant_range) = packet_range.intersect(address_range) {
                    let dmx_data = output.data.as_ref();
                    // TODO: Once the Step trait is stabilitized, we could
                    // implement it for DmxAddress and iterate relevant_range directly
                    for address_value in relevant_range.start.value()..relevant_range.end.value() {
                        let address = DmxAddress::from(address_value);
                        let index = self.frame_index_of(address);
                        self.frame[index] = dmx_data[address.channel() as usize];
                    }
                    self.update_lighthouse().await?;
                }
            },
            _ => info! {
                ?command,
                "Ignoring"
            },
        }

        Ok(())
    }

    fn address_range(&self) -> Range<DmxAddress> {
        self.start_address..self.end_address()
    }

    fn end_address(&self) -> DmxAddress {
        self.start_address + (LIGHTHOUSE_BYTES as u32)
    }

    fn frame_index_of(&self, dmx_address: DmxAddress) -> usize {
        (dmx_address - self.start_address).value() as usize
    }

    async fn update_lighthouse(&mut self) -> Result<()> {
        self.lh.put_model(self.frame.into()).await?;
        Ok(())
    }
}
