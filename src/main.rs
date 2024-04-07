use anyhow::Result;
use artnet_protocol::ArtCommand;
use clap::Parser;
use lighthouse_client::{protocol::Authentication, Lighthouse, LIGHTHOUSE_URL};
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

    let socket = UdpSocket::bind(("0.0.0.0", 6454)).await?;

    info!("Listening for Art-Net packets on {} (UDP)", socket.local_addr()?);
    loop {
        // TODO: Handle errors
        let mut buffer = [0u8; 1024];
        let (length, addr) = socket.recv_from(&mut buffer).await?;
        let command = ArtCommand::from_buffer(&buffer[..length])?;

        debug!(%addr, ?command, "Received command");
    }
}
