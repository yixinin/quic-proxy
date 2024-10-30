use std::net::SocketAddr;

pub struct Endpoint {}

// impl Endpoint {
//     fn new() -> Endpoint {
//         Endpoint {}
//     }

//     async fn connect() -> Result<()> {
//         let cfg = quinn::ServerConfig::new(crypto, token_key);
//         let server = quinn::Endpoint::server(cfg, SocketAddr::try_from(":8080")?)?;
//         while let Some(conn) = server.accept().await {
//             let conn = conn.await?;
//             let (tx, rx) = conn.accept_bi().await?;
//             tx.write
//         }
//     }
// }
