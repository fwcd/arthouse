use std::{sync::{atomic::{AtomicUsize, Ordering}, Arc}, time::{Duration, Instant}};

use anyhow::Result;
use artnet_protocol::ArtCommand;
use lighthouse_client::{protocol::LIGHTHOUSE_BYTES, Lighthouse, TokioWebSocket};
use tokio::{net::UdpSocket, sync::mpsc};
use tracing::{debug, info, info_span, warn, Instrument};

use crate::{address::DmxAddress, allocation::DmxAllocation};

pub struct ArtNetAdapter {
    lh: Option<Lighthouse<TokioWebSocket>>,
    socket: UdpSocket,
    allocation: DmxAllocation,
    frame: [u8; LIGHTHOUSE_BYTES],
    dropped_frames: Arc<AtomicUsize>,
    sender: mpsc::Sender<[u8; LIGHTHOUSE_BYTES]>,
    receiver: Option<mpsc::Receiver<[u8; LIGHTHOUSE_BYTES]>>,
}

impl ArtNetAdapter {
    pub fn new(lh: Lighthouse<TokioWebSocket>, socket: UdpSocket, allocation: DmxAllocation) -> Self {
        let (sender, receiver) = mpsc::channel(1);
        Self { lh: Some(lh), socket, allocation, frame: [0u8; LIGHTHOUSE_BYTES], dropped_frames: Arc::new(AtomicUsize::new(0)), sender, receiver: Some(receiver) }
    }

    pub async fn run(mut self) -> Result<()> {
        // TODO: Factor out Lighthouse forwarder into separate structure?
        let mut lh = self.lh.take().unwrap();
        let mut receiver = self.receiver.take().unwrap();
        let dropped_frames = self.dropped_frames.clone();
        let mut last_warning = Instant::now();
        tokio::spawn(async move {
            while let Some(frame) = receiver.recv().await {
                let result = lh.put_model(frame.into()).await;
                if let Err(e) = result {
                    warn!(error = %e, "Error while sending frame to lighthouse")
                }
                if last_warning.elapsed() >= Duration::from_secs(1) {
                    let dropped = dropped_frames.fetch_and(0, Ordering::Relaxed);
                    if dropped > 0 {
                        warn!(dropped_frames = dropped, "Lighthouse cannot keep up");
                        last_warning = Instant::now();
                    }
                }
            }
        });

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
                let universe = u16::from(output.port_address) as usize;
                debug! {
                    version = ?output.version,
                    sequence = output.sequence,
                    universe = universe,
                    length = *output.length,
                    "Handling output"
                };
                let range = self.allocation.address_range_in(universe);
                if !range.is_empty() {
                    let dmx_data = output.data.as_ref();
                    // TODO: Once the Step trait is stabilitized we could
                    // implement it for DmxAddress and make the range itself
                    // iterable.
                    for value in range.start.value()..range.end.value() {
                        let address = DmxAddress::from(value);
                        let index = self.allocation.index_of(address).unwrap();
                        self.frame[index] = dmx_data[address.channel()];
                    }
                    if let Err(_) = self.sender.try_send(self.frame) {
                        self.dropped_frames.fetch_add(1, Ordering::Relaxed);
                    }
                }
            },
            _ => info! {
                ?command,
                "Ignoring"
            },
        }

        Ok(())
    }
}
