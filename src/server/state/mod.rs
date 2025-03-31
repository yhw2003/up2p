pub mod get {
    use std::sync::{Arc, LazyLock};
    use tokio::net::UdpSocket;
    use tracing::error;
    use ostatu_rs::{AppState, GetState};

    use crate::ServerConfig;

    pub fn get_udp_socket() -> Arc<UdpSocket> {
        match AppState::get_state(None) {
            Some(udp_socket) => udp_socket,
            None => {
                error!("Failed to get udp socket");
                panic!();
            }
        }
    }
    pub fn get_server_config() -> crate::ServerConfig {
        match AppState::get_state(None) {
            Some(server_config) => server_config,
            None => {
                error!("Failed to get server config");
                panic!();
            }
        }
    }
    pub fn get_identitier() -> Arc<dyn Fn(&str) -> bool> {
        static SERVER_CONFIG: LazyLock<ServerConfig> = LazyLock::new(|| {
            get_server_config()
        });
        let identitier: Arc<dyn Fn(&str) -> bool> = Arc::new(|identity| {
            identity == SERVER_CONFIG.identity
        });
        identitier
    }
}


pub mod set {
    use std::sync::Arc;
    use tokio::net::UdpSocket;
    use tracing::error;
    use ostatu_rs::{AppState, GetState};

    pub fn set_udp_socket(udp_socket: Arc<UdpSocket>) {
        if let Err(e)  = AppState::set_state(None, udp_socket) {
            error!("Failed to set udp socket, {}", e);
        };
    }
    pub fn set_server_config(server_config: crate::ServerConfig) {
        if let Err(e)  = AppState::set_state(None, server_config) {
            error!("Failed to set udp socket, {}", e);
        };
    }
}
