use anyhow::Result;
use quinn_proto::crypto::rustls::QuicServerConfig;
use std::{
    fs, io,
    net::{SocketAddr, UdpSocket},
    sync::Arc,
};
use tokio::{io::AsyncWriteExt, net::TcpStream};

pub struct TCPClient {
    addr: SocketAddr,
    lis: quinn::Endpoint,
}

impl TCPClient {
    pub fn new(
        addr: SocketAddr,
        key_path: String,
        cert_path: String,
        socket: UdpSocket,
    ) -> Result<TCPClient> {
        let key = fs::read(key_path)?; //.context("failed to read private key")?;
        let key = rustls_pemfile::private_key(&mut &*key)?
            .ok_or_else(|| anyhow::Error::msg("no private keys found"))?;

        let certs = fs::read(cert_path)?; //.context("failed to read certificate chain")?;
        let certs = rustls_pemfile::certs(&mut &*certs).collect::<Result<_, _>>()?;
        let server_crypto = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)?;
        let mut server_config =
            quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(server_crypto)?));
        let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
        transport_config.max_concurrent_uni_streams(0_u8.into());

        let runtime = quinn::default_runtime()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no async runtime found"))?;
        let socket = runtime.wrap_udp_socket(socket)?;
        let endpoint = quinn::Endpoint::new_with_abstract_socket(
            quinn::EndpointConfig::default(),
            Some(server_config),
            socket,
            runtime,
        )?;
        eprintln!("bp tcp listening on {}", endpoint.local_addr()?);
        Ok(TCPClient {
            addr: addr,
            lis: endpoint,
        })
    }

    pub async fn listen(&self) -> Result<()> {
        loop {
            let addr = self.addr.clone();
            if let Some(incom) = self.lis.accept().await {
                eprintln!("bp quic incoming new conn");
                tokio::spawn(async move {
                    if let Ok(conn) = incom.await {
                        eprintln!("bp quic incoming new conn: {}", conn.remote_address());

                        if let Ok((mut tx, mut rx)) = conn.accept_bi().await {
                            eprintln!("bp quic incoming new stream: {}", conn.remote_address());

                            eprintln!("bp tcp try connecting to {}", addr.clone());
                            if let Ok(mut stream) = TcpStream::connect(addr.clone()).await {
                                let (mut rrx, mut rtx) = stream.split();

                                eprintln!("bp tcp connected to {}, start copy", addr.clone());
                                let x1 = tokio::io::copy(&mut rrx, &mut tx);
                                let x2 = tokio::io::copy(&mut rx, &mut rtx);
                                tokio::select! {
                                    _= x1=>{
                                        eprintln!("bp copy x1 end")
                                    },
                                    _= x2=>{
                                        eprintln!("bp copy x2 end")
                                    },
                                };

                                let _ = stream.shutdown().await;
                                let _ = tx.flush().await;
                                let _ = tx.finish();
                            }
                        }
                    }
                });
            }
        }
    }
}
