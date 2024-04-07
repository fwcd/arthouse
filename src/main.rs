mod adapter;

use std::{net::SocketAddr, str::FromStr};

use adapter::ArtNetAdapter;
use anyhow::Result;
use clap::Parser;
use lighthouse_client::{protocol::Authentication, Lighthouse, LIGHTHOUSE_URL};
use socket2::{Domain, Socket, Type};
use tokio::net::UdpSocket;

#[derive(Parser)]
#[command(version)]
struct Args {
    /// The username.
    #[arg(short, long, env = "LIGHTHOUSE_USER")]
    username: String,
    /// The API token.
    #[arg(short, long, env = "LIGHTHOUSE_TOKEN")]
    token: String,
    /// The server URL.
    #[arg(long, env = "LIGHTHOUSE_URL", default_value = LIGHTHOUSE_URL)]
    url: String,
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

    let adapter = ArtNetAdapter::new(lh, tokio_socket);
    adapter.run().await
}
