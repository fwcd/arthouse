mod adapter;
mod address;

use std::{net::SocketAddr, str::FromStr};

use adapter::ArtNetAdapter;
use address::DmxAddress;
use anyhow::Result;
use clap::Parser;
use lighthouse_client::{protocol::Authentication, Lighthouse, LIGHTHOUSE_URL};
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
    /// The first DMX universe (Art-Net port address) to use.
    #[arg(short, long, env = "ARTHOUSE_UNIVERSE", default_value_t = 0)]
    universe: u16,
    /// The first DMX channel to use.
    #[arg(short, long, env = "ARTHOUSE_CHANNEL", default_value_t = 0)]
    channel: u8,
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
    s2_socket.bind(&SocketAddr::from_str("0.0.0.0:6454")?.into())?;

    let std_socket = std::net::UdpSocket::from(s2_socket);
    let tokio_socket = UdpSocket::from_std(std_socket)?;

    let start_address = DmxAddress::new(args.universe, args.channel);
    let adapter = ArtNetAdapter::new(lh, tokio_socket, start_address);
    adapter.run().await
}
