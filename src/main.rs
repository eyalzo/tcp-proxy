use clap::{Parser};
use tokio::io;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use std::os::unix::io::AsRawFd;
use nix::sys::socket::setsockopt;
use nix::sys::socket::sockopt::IpTransparent;
use tokio::select;

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    /// The full IP:port address of the traffic we listen to
    #[clap(short, long, value_parser)]
    client: SocketAddr,
    /// The full IP:port address of the destination where we send the traffic to
    #[clap(short, long, value_parser, required=true)]
    server: SocketAddr,
}

async fn proxy(client: SocketAddr, server: SocketAddr) -> io::Result<()> {
    let listener = TcpListener::bind(client).await?;
    // Linux only- get raw socket for setsockopt()
    let raw_socket = listener.as_raw_fd();
    let _res = setsockopt(raw_socket, IpTransparent, &true);

    loop {
        let (client, _) = listener.accept().await?;
        let server = TcpStream::connect(server).await?;

        let (mut eread, mut ewrite) = client.into_split();
        let (mut oread, mut owrite) = server.into_split();

        let e2o = tokio::spawn(async move { io::copy(&mut eread, &mut owrite).await });
        let o2e = tokio::spawn(async move { io::copy(&mut oread, &mut ewrite).await });

        // let e2o = io::copy(&mut eread, &mut owrite);
        // let o2e = io::copy(&mut oread, &mut ewrite);

        select! {
                _ = e2o => println!("c2s done"),
                _ = o2e => println!("s2c done"),

        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Cli::parse();

    proxy(args.client, args.server).await
}