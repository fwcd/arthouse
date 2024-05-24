use std::{sync::Arc, time::{Duration, Instant}};

use anyhow::Result;
use artnet_protocol::ArtCommand;
use lighthouse_client::{protocol::LIGHTHOUSE_BYTES, Lighthouse, TokioWebSocket};
use tokio::{net::UdpSocket, sync::{Mutex, Notify}};
use tracing::{debug, info, info_span, warn, Instrument};

use crate::{address::DmxAddress, allocation::DmxAllocation};

pub struct ArtNetAdapter {
    lh: Option<Lighthouse<TokioWebSocket>>,
    socket: UdpSocket,
    allocation: DmxAllocation,
    frame_state: Arc<Mutex<FrameState>>,
    notify: Arc<Notify>,
}

#[derive(Clone, Copy)]
struct FrameState {
    frame: [u8; LIGHTHOUSE_BYTES],
    updates: usize,
}

impl ArtNetAdapter {
    pub fn new(lh: Lighthouse<TokioWebSocket>, socket: UdpSocket, allocation: DmxAllocation) -> Self {
        Self {
            lh: Some(lh),
            socket,
            allocation, 
            frame_state: Arc::new(Mutex::new(FrameState {
                frame: [0u8; LIGHTHOUSE_BYTES],
                updates: 0,
            })),
            notify: Arc::new(Notify::new()),
        }
    }

    pub async fn run(mut self) -> Result<()> {
        self.spawn_lighthouse_forwarder();
        self.run_artnet_listener().await
    }

    fn spawn_lighthouse_forwarder(&mut self) {
        let mut lh = self.lh.take().unwrap();
        let frame_state = self.frame_state.clone();
        let notify = self.notify.clone();
        tokio::spawn(async move {
            let mut last_second = Instant::now();
            let mut dropped_frames = 0;
            loop {
                notify.notified().await;
                let frame = {
                    let mut frame_state = frame_state.lock().await;
                    if frame_state.updates == 0 {
                        continue;
                    }
                    dropped_frames += frame_state.updates - 1;
                    frame_state.updates = 0;
                    frame_state.frame
                };
                let result = lh.put_model(frame.into()).await;
                if let Err(e) = result {
                    warn!(error = %e, "Error while sending frame to lighthouse")
                }
                if last_second.elapsed() >= Duration::from_secs(1) {
                    if dropped_frames > 0 {
                        warn!(dropped_frames, "Lighthouse cannot keep up");
                        last_second = Instant::now();
                        dropped_frames = 0;
                    }
                }
            }
        });
    }

    async fn run_artnet_listener(mut self) -> Result<()> {
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
                    let mut changed = false;
                    {
                        let mut frame_state = self.frame_state.lock().await;
                        // TODO: Once the Step trait is stabilitized we could
                        // implement it for DmxAddress and make the range itself
                        // iterable.
                        for value in range.start.value()..range.end.value() {
                            let address = DmxAddress::from(value);
                            let index = self.allocation.index_of(address).unwrap();
                            changed |= frame_state.frame[index] != dmx_data[address.channel()];
                            frame_state.frame[index] = dmx_data[address.channel()];
                        }
                        if changed {
                            frame_state.updates += 1;
                        }
                    }
                    if changed {
                        self.notify.notify_one();
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
