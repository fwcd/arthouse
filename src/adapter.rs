use anyhow::Result;
use artnet_protocol::ArtCommand;
use lighthouse_client::{Lighthouse, TokioWebSocket};
use tokio::net::UdpSocket;
use tracing::{info, info_span, warn, Instrument};

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

    async fn handle_raw(&self, raw: &[u8]) -> Result<()> {
        let command = ArtCommand::from_buffer(raw)?;
        self.handle_command(command).await
    }

    async fn handle_command(&self, command: ArtCommand) -> Result<()> {
        info!(?command, "Handling command");
        Ok(())
    }
}
