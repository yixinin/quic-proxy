use anyhow::Result;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
pub mod bp;
pub mod fp;
pub mod tunnel;
#[tokio::main]
async fn main() -> std::io::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let key_path = String::from("quic.key");
    let cert_path = String::from("quic.crt");
    let server_name = String::from("local.com");
    let laddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9999);
    let raddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9080);
    let x1 = tokio::spawn(async move {
        if let Err(err) = start_server(raddr, key_path, cert_path).await {
            println!("server listen end with err: {}, exit...", err)
        } else {
            println!("server listen end, exit...")
        }
    });
    let x2 = tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        if let Err(err) = start_client(laddr, server_name).await {
            println!("client listen end with err: {}, exit...", err)
        } else {
            println!("client listen end, exit...")
        }
    });

    tokio::join!(x1, x2);
    Ok(())
}

async fn start_client(laddr: SocketAddr, server_name: String) -> Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:7777")?;
    socket.connect("127.0.0.1:9999")?;
    let client = fp::tcp::TCPServer::new(laddr, server_name, socket)?;
    client.listen().await?;
    Ok(())
}

async fn start_server(raddr: SocketAddr, key_path: String, cert_path: String) -> Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:9999")?;

    let server = bp::tcp::TCPClient::new(raddr, key_path, cert_path, socket)?;
    server.listen().await?;
    Ok(())
}
