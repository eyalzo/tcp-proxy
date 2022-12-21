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

async fn proxy(client_addr: SocketAddr, server_addr: SocketAddr) -> io::Result<()> {
    let listener = TcpListener::bind(client_addr).await?;
    println!("Bind successfully on {}", listener.local_addr().unwrap());
    // Linux only- get raw socket for setsockopt()
    let raw_socket = listener.as_raw_fd();
    let _res = setsockopt(raw_socket, IpTransparent, &true);

    loop {
        println!("Waiting for a client to connect {} ...", listener.local_addr().unwrap());
        let (client, _) = listener.accept().await?;
        println!("   Client connected from {}", client.local_addr().unwrap());

        println!("Trying to connect server on {} ...", server_addr);
        let server = TcpStream::connect(server_addr).await?;
        println!("   Server {} connected", server.peer_addr().unwrap());

        let (mut c_read, mut c_write) = client.into_split();
        let (mut s_read, mut s_write) = server.into_split();

        let c2s = tokio::spawn(async move {
            let c2s_count = io::copy(&mut c_read, &mut s_write).await;
            println!("Copied client -> server: {} bytes", c2s_count.unwrap());
        });

        let s2c = tokio::spawn(async move {
            let s2c_count = io::copy(&mut s_read, &mut c_write).await;
            println!("Copied server -> client: {} bytes", s2c_count.unwrap());
        });

        // let e2o = io::copy(&mut eread, &mut owrite);
        // let o2e = io::copy(&mut oread, &mut ewrite);

        select! {
                _ = c2s => println!("c2s done"),
                _ = s2c => println!("s2c done"),

        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Cli::parse();

    proxy(args.client, args.server).await
}