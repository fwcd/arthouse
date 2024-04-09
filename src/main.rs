mod adapter;
mod address;
mod allocation;
mod constants;
mod utils;

use std::{net::{IpAddr, SocketAddr}, str::FromStr};

use adapter::ArtNetAdapter;
use address::DmxAddress;
use allocation::DmxAllocation;
use anyhow::Result;
use clap::Parser;
use lighthouse_client::{protocol::{Authentication, LIGHTHOUSE_BYTES, LIGHTHOUSE_COLS}, Lighthouse, LIGHTHOUSE_URL};
use socket2::{Domain, Socket, Type};
use tokio::net::UdpSocket;

#[derive(Parser)]
#[command(version, after_help =
    "Note: Lighthouse frames occupy 14 * 28 * 3 = 1176 channels, therefore three consecutive universes are used."
)]
struct Args {
    /// The Project Lighthouse username.
    #[arg(long, env = "LIGHTHOUSE_USER")]
    username: String,
    /// The Project Lighthouse API token.
    #[arg(long, env = "LIGHTHOUSE_TOKEN")]
    token: String,
    /// The Project Lighthouse server URL.
    #[arg(long, env = "LIGHTHOUSE_URL", default_value = LIGHTHOUSE_URL)]
    url: String,
    /// The host on which to bind the Art-Net/UDP socket.
    #[arg(long, env = "ARTHOUSE_HOST", default_value = "0.0.0.0")]
    host: String,
    /// The port on which to bind the Art-Net/UDP socket.
    #[arg(long, env = "ARTHOUSE_PORT", default_value_t = 6454)]
    port: u16,
    /// The first DMX universe (Art-Net port address) to use.
    #[arg(short, long, env = "ARTHOUSE_UNIVERSE", default_value_t = 0)]
    universe: usize,
    /// The first DMX channel to use.
    #[arg(short, long, env = "ARTHOUSE_CHANNEL", default_value_t = 0)]
    channel: usize,
    /// Group size for which channels will not be split across universes.
    #[arg(short, long, env = "ARTHOUSE_GROUPING", default_value_t = LIGHTHOUSE_COLS * 3)]
    grouping: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    _ = dotenvy::dotenv();

    let args = Args::parse();
    let auth = Authentication::new(&args.username, &args.token);
    let lh = Lighthouse::connect_with_tokio(auth).await?;

    let s2_socket = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
    s2_socket.set_broadcast(true)?;
    s2_socket.set_reuse_port(true)?;
    s2_socket.bind(&SocketAddr::from((IpAddr::from_str(&args.host)?, args.port)).into())?;

    let std_socket = std::net::UdpSocket::from(s2_socket);
    let tokio_socket = UdpSocket::from_std(std_socket)?;

    let start_address = DmxAddress::new(args.universe, args.channel);
    let allocation = DmxAllocation::new(start_address, LIGHTHOUSE_BYTES, args.grouping);
    let adapter = ArtNetAdapter::new(lh, tokio_socket, allocation);
    adapter.run().await
}
