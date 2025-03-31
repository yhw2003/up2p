mod services;
mod state;
mod base;

use std::{str::FromStr, sync::Arc};

use services::{event_router, udp_event_handle::Up2pEvent};
use state::set::{set_server_config, set_udp_socket};
use tokio::{net::UdpSocket, signal};
use serde::Deserialize;
// use tokio::net::UdpSocket;
use tracing::{info, warn, Level};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_config = ServerConfig::parse_config()?;
    // init logger
    tracing_subscriber::fmt().with_max_level(Level::from_str(&server_config.log_level)?).init();
    info!("Starting server...");
    // open udp socket


    let udp_socket = Arc::new(UdpSocket::bind(
            &format!("{}:{}", server_config.address, server_config.port)
    ).await?);


    set_udp_socket(udp_socket);
    set_server_config(server_config);

    let (rx, handle) = services::udp_event_handle().await?;
    let mut app = ServerApp::new(rx);
    tokio::select! {
        _ = signal::ctrl_c() => {
            handle.abort();
            info!("Shutting down server...");
        }
        _ = app.run() => {
            handle.abort();
            warn!("Shutting down server Unexpectly...");
        }
    }
    Ok(())
}

#[derive(Debug)]
struct ServerApp {
    rx: tokio::sync::mpsc::Receiver<Up2pEvent>,
}

impl ServerApp {
    pub fn new(rx: tokio::sync::mpsc::Receiver<Up2pEvent>) -> Self {
        Self {
            rx,
        }

    }
    pub async fn run(&mut self) -> anyhow::Result<()> {
        loop {
            if let Some(event) = self.rx.recv().await {
                event_router::route(event).await;
            } else {
                warn!("Recieved None from the channel");
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ServerConfig {
    address: String,
    port: u16,
    log_level: String,
    identity: String,
}

impl ServerConfig {
    fn parse_toml(toml_str: &str) -> anyhow::Result<Self> {
        let config = toml::from_str(toml_str)?;
        Ok(config)
    }

    fn parse_config() -> anyhow::Result<Self> {
        let config = std::fs::read_to_string("up2pd.toml")?;
        Self::parse_toml(&config)
    }
}