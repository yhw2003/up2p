use std::{str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;
use tracing::Level;
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
    // let _hello_pkg = up2p_client.0.client_hello().await?;
    let endpoint = up2p_client.0.client_request(
        RequestInfo {
            client_class: "cli".to_string(),
            client_instance: _client_instance_config.client_instance,
        }
    ).await.unwrap().unwrap();
    println!("hello pkg: {:?}", endpoint);
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
        let config = std::fs::read_to_string("up2pc.toml")?;
        Self::parse_toml(&config)
    }
}