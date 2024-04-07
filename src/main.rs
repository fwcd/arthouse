use std::{net::SocketAddr, str::FromStr};

use anyhow::Result;
use artnet_protocol::ArtCommand;
use clap::Parser;
use lighthouse_client::{protocol::Authentication, Lighthouse, LIGHTHOUSE_URL};
use socket2::{Domain, Socket, Type};
use tokio::net::UdpSocket;
use tracing::{debug, info};

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

    info!("Listening for Art-Net packets on {} (UDP)", tokio_socket.local_addr()?);
    loop {
        // TODO: Handle errors
        let mut buffer = [0u8; 1024];
        let (length, addr) = tokio_socket.recv_from(&mut buffer).await?;
        let command = ArtCommand::from_buffer(&buffer[..length])?;

        debug!(%addr, ?command, "Received command");
    }
}
