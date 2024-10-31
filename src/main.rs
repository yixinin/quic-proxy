use anyhow::Result;
use config::Config;
use std::{
    fs,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    str::FromStr,
};
pub mod bp;
pub mod config;
pub mod fp;
pub mod tls;
pub mod tunnel;

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let data = fs::read_to_string("config.toml")?;
    let cfg: Config = toml::from_str(&data)?;
    eprintln!("read config:\n {:#?}", cfg);
    let raddr = SocketAddr::from_str(&cfg.backend.proxy_pass)?;
    let laddr = SocketAddr::from_str(&cfg.frontend.listen)?;

    let x1 = tokio::spawn(async move {
        if let Err(err) = start_server(
            raddr,
            cfg.backend.ssl_certificate_key,
            cfg.backend.ssl_certificate,
        )
        .await
        {
            println!("server listen end with err: {}, exit...", err)
        } else {
            println!("server listen end, exit...")
        }
    });
    let x2 = tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        if let Err(err) = start_client(laddr, cfg.frontend.server_name).await {
            println!("client listen end with err: {}, exit...", err)
        } else {
            println!("client listen end, exit...")
        }
    });
    tokio::select! {
        _=x1 =>{
            eprintln!("x1 done, exit.");
        },
        _=x2 =>{
            eprintln!("x2 done, exit.");
        }
    };
    eprintln!("all done, exit.");
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
