use std::{sync::Arc, time::Duration};

use bincode::config;
use serde::{Deserialize, Serialize};
use tokio::{net::UdpSocket, time};
use up2p::core::{bincodec::BinCodec, uprotocol_pkg::{ClientHelloPkg, ClientRequestPkg}, BaseUp2pProtocol};

pub const CLIENT_CLASS: &str = "demo_up2pc";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // client hello
    let udp_socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await.unwrap());
    let client_config = ClientConfig::parse_config()?;
    let test_protocol_data = BaseUp2pProtocol::client_hello_with_payload(
        ClientHelloPkg::new(CLIENT_CLASS, &client_config.client_instance, &client_config.identity, 0x1)
    ).unwrap();
    
    let data = bincode::encode_to_vec(&test_protocol_data, config::standard()).unwrap();
    udp_socket.send_to(&data, &client_config.server_address).await.unwrap();
    time::sleep(Duration::from_secs(1)).await;

    // client request
    let test_protocol_data = BaseUp2pProtocol::request_with_payload(
        ClientRequestPkg::create_endpoint_request(
            CLIENT_CLASS, &client_config.client_instance, &client_config.identity, &up2p::utils::get_global_id(CLIENT_CLASS, &client_config.client_instance)
        )
    )?;
    let uu = udp_socket.clone();
    let handle = tokio::spawn(async move {
        let mut buf = vec![0; 1024];
        let (len, _) = uu.recv_from(&mut buf).await.unwrap();
        let base_protocol = BaseUp2pProtocol::decode_from(&buf[..len]).unwrap();
        println!("Received: {:?}", base_protocol);
        let s = String::from_utf8(base_protocol.get_payload().to_vec()).unwrap();
        println!("Received: {:?}", s);
    });
    time::sleep(Duration::from_secs(1)).await;
    let data = bincode::encode_to_vec(&test_protocol_data, config::standard()).unwrap();
    udp_socket.send_to(&data, client_config.server_address).await.unwrap();
    handle.await.unwrap();
    Ok(())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ClientConfig {
    client_instance: String,
    identity: String,
    server_address: String,
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