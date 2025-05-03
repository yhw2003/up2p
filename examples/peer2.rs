use std::{str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;
use tracing::{info, Level};
use up2p::{client_lib::app::Up2pCli, core::{request_info::RequestInfo, uprotocol_pkg::BasePkg}};

#[tokio::main]
async fn main () -> anyhow::Result<()> {
    let client_config = ClientConfig::parse_config().unwrap();
    let _client_instance_config = client_config.clone();
    tracing_subscriber::fmt().with_max_level(Level::from_str(&client_config.log_level)?).init();
    let udp_socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await.unwrap());
    let server_address = client_config.server_address.parse::<std::net::SocketAddr>().unwrap();
    let up2p_client = Up2pCli::new(BasePkg {
        client_instance: client_config.client_instance,
        client_class: "cli".to_string(),
        identity: client_config.identity,
    }, udp_socket, (server_address.ip(), server_address.port()));
    up2p_client.0.start().await.unwrap();
    up2p_client.0.client_hello().await?;
    let r = up2p_client.0.client_request(RequestInfo { client_class: "cli".to_string(), client_instance: "peer1".to_string() }).await?.unwrap();
    let _result = up2p_client.0.pkg_send_to(
        // vec 255-0
        client_config.server_address.parse().unwrap(), Vec::from_iter(0..255u8),
        Some(BasePkg {
            client_instance: "peer1".to_string(),
            client_class: "cli".to_string(),
            identity: "bbb".to_string(),
        })
    ).await?;
    info!("endpoint: {:?}", r);
    Ok(())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ClientConfig {
    client_instance: String,
    identity: String,
    server_address: String,
    log_level: String,
}

impl ClientConfig {
    pub fn parse_toml(toml: &str) -> anyhow::Result<Self> {
        let config: Self = toml::from_str(toml)?;
        Ok(config)
    }
    pub fn parse_config() -> anyhow::Result<Self> {
        let config = std::fs::read_to_string("peer2.toml")?;
        Self::parse_toml(&config)
    }
}