use std::{
    io,
    net::{SocketAddr, UdpSocket},
    sync::Arc,
};

use anyhow::Result;

use quinn_proto::crypto::rustls::QuicClientConfig;
use tokio::io::AsyncWriteExt;

use crate::tls;
pub struct TCPServer {
    laddr: SocketAddr,
    raddr: SocketAddr,
    server_name: String,
    client: quinn::Endpoint,
}

impl TCPServer {
    pub fn new(laddr: SocketAddr, server_name: String, socket: UdpSocket) -> Result<Self> {
        let raddr = socket.peer_addr()?;
        let client_crypto = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(tls::DebugVerify {}))
            .with_no_client_auth();

        let qccfg = QuicClientConfig::try_from(client_crypto)?;

        let client_config = quinn::ClientConfig::new(Arc::new(qccfg));

        let runtime = quinn::default_runtime()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no async runtime found"))?;

        let socket = runtime.wrap_udp_socket(socket)?;
        let mut client = quinn::Endpoint::new_with_abstract_socket(
            quinn::EndpointConfig::default(),
            None,
            socket,
            runtime,
        )?;
        client.set_default_client_config(client_config);

        Ok(TCPServer {
            laddr: laddr,
            raddr: raddr,
            server_name: server_name,
            client: client,
        })
    }

    pub async fn listen(&self) -> io::Result<()> {
        eprintln!("fp quic try connecting to {}", self.raddr);
        let conn = match self.client.connect(self.raddr, &self.server_name) {
            Ok(conn) => Ok(conn.await?),
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionAborted,
                e.to_string(),
            )),
        }?;
        eprintln!("fp quic connected to {}", self.raddr);
        let lis = tokio::net::TcpListener::bind(self.laddr).await?;
        eprintln!("fp tcp listening on {}", self.laddr);
        loop {
            if let Ok((mut stream, _raddr)) = lis.accept().await {
                eprintln!("fp tcp accept new stream {}, quic open bi", self.laddr);
                if let Ok((mut tx, mut rx)) = conn.open_bi().await {
                    tokio::task::spawn(async move {
                        eprintln!("fp tcp start copy");
                        let (mut srx, mut stx) = stream.split();
                        let x1 = tokio::io::copy(&mut rx, &mut stx);
                        let x2 = tokio::io::copy(&mut srx, &mut tx);
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
                    });
                }
            }
        }
    }
}
