use anyhow::Result;
use artnet_protocol::ArtCommand;
use lighthouse_client::{Lighthouse, TokioWebSocket};
use tokio::net::UdpSocket;
use tracing::{debug, info};

pub struct ArtNetAdapter {
    lh: Lighthouse<TokioWebSocket>,
    socket: UdpSocket,
}

impl ArtNetAdapter {
    pub fn new(lh: Lighthouse<TokioWebSocket>, socket: UdpSocket) -> Self {
        Self { lh, socket }
    }

    pub async fn run(self) -> Result<()> {
        info!("Listening for Art-Net packets on {} (UDP)", self.socket.local_addr()?);

        loop {
            // TODO: Handle errors
            let mut buffer = [0u8; 1024];
            let (length, addr) = self.socket.recv_from(&mut buffer).await?;
            let command = ArtCommand::from_buffer(&buffer[..length])?;

            debug!(%addr, ?command, "Received command");
        }
    }
}
