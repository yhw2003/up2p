use std::{collections::HashMap, net::SocketAddr, sync::{Arc, LazyLock}};

use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use up2p::core::{bincodec::BinCodec, get_global_id::GetGlobalId, uprotocol_pkg::{ClientHelloPkg, ClientRequestAckPkg, ClientRequestPkg, GetBaseInfo, PeerExchangePkg, PkgVerifyIdentity}, BaseUp2pProtocol};

use super::udp_event_handle::Up2pEvent;

// static DEVICE_LIST: LazyLock<Arc<Mutex<HashMap<String, SocketAddr>>>> = LazyLock::new(|| {
//     Arc::new(Mutex::new(HashMap::new()))
// });

static DEVICE_LIST: LazyLock<Arc<RwLock<HashMap<String, SocketAddr>>>> = LazyLock::new(|| {
    Arc::new(RwLock::new(HashMap::new()))
});

pub async fn route(event: Up2pEvent) {
    let ubase_protocal_pkg = event.get_data();
    let base_bind_result = BaseUp2pProtocol::decode_from(&ubase_protocal_pkg);
    match base_bind_result {
        Err(e) => {
            warn!("Failed to decode base protocal package: {:?}", e);
        },
        Ok(base_protocal) => {
            info!("Recieved a base protocal package: {:?}", base_protocal);
            match base_protocal.get_pkg_type() {
                BaseUp2pProtocol::TYPE_HELLO => {
                    if let Err(e) = handle_client_hello_pkg(base_protocal.get_payload(), event.get_addr()).await {
                        warn!("Failed to handle client hello package: {:?}", e);
                    };
                },
                BaseUp2pProtocol::TYPE_REQUEST => {
                    // handle request
                    if let Err(e) = handle_client_request_pkg(base_protocal.get_payload(), event.get_addr()).await {
                        warn!("Failed to handle client request package: {:?}", e);
                    };
                },
                BaseUp2pProtocol::TYPE_PKG_EXCHANGE => {
                    let _payload = base_protocal.get_payload();
                    if let Err(e) = handle_exchange_pkg(base_protocal.get_payload(), event.get_addr()).await {
                        warn!("Failed to handle client hello package: {:?}", e);
                    };
                },
                _ => {
                    warn!("Unkown base protocal type: {:?}", base_protocal);
                }
            }
        }
    }
}

async fn handle_client_hello_pkg(payload: &[u8], endpoint_addr: SocketAddr) -> anyhow::Result<()> {
    let server_config = crate::state::get::get_server_config();
    if let Ok((clien_hello_pkg, _size)) = bincode::decode_from_slice::<
        ClientHelloPkg, bincode::config::Configuration
    >(payload, up2p::get_binencode_config()) {
        clien_hello_pkg.verify_identity(&server_config.identity)?;
        match clien_hello_pkg.get_msg() {
            ClientHelloPkg::MSG_HELLO => {
                info!("Client hello: {}", endpoint_addr);
                // Add the device to the device list
                DEVICE_LIST.write().await.insert(clien_hello_pkg.get_global_id(), endpoint_addr);
                let udp_socket = crate::state::get::get_udp_socket();
                let pp = BaseUp2pProtocol::hello_ack_with_payload()?;
                let encoded = pp.encode_to_vec()?;
                debug!("Encoded response: {:?}", encoded);
                udp_socket.send_to(&encoded, endpoint_addr).await?;
                debug!("Device list: {:?}", DEVICE_LIST.read().await);
            },
            _ => warn!("Unkown client hello message: {:?}", clien_hello_pkg)
        }
    } else {
        warn!("Failed to decode client hello package");
    };
    Ok(())
}

async fn handle_client_request_pkg(payload: &[u8], endpoint_addr: SocketAddr) -> anyhow::Result<()> {
    let udp_socket = crate::state::get::get_udp_socket();
    let server_config = crate::state::get::get_server_config();
    if let Ok((client_request_pkg, _size)) = bincode::decode_from_slice::<
        ClientRequestPkg, bincode::config::Configuration
    >(payload, up2p::get_binencode_config()) {
        client_request_pkg.verify_identity(&server_config.identity)?;
        match client_request_pkg.get_request_type() {
            ClientRequestPkg::REQUEST_ENDPOINT => {
                info!("Client request endpoint: {}", endpoint_addr);
                // Add the device to the device list
                let requested_global_id = client_request_pkg.get_payload_as_global_id()?;
                if let Some(ov) = DEVICE_LIST.read().await.get(&requested_global_id) {
                    let pp = BaseUp2pProtocol::response_with_payload(ClientRequestAckPkg::new(format!("{}:{}", ov.ip(), ov.port())))?;
                    let encoded = bincode::encode_to_vec(&pp, up2p::get_binencode_config())?;
                    udp_socket.send_to(&encoded, endpoint_addr).await?;
                } else {
                    warn!("Requested device not found: {}", requested_global_id);
                };
                debug!("Device list: {:?}", DEVICE_LIST.read().await);
            },
            _=> warn!("Unkown client request type: {:?}", client_request_pkg)
        }
    } else {
        warn!("Failed to decode client request package, payload: {:?}", payload);
    }
    Ok(())
}

async fn handle_exchange_pkg(payload: &[u8], _endpoint_addr: SocketAddr) -> anyhow::Result<()> {
    let exchange_pkg = PeerExchangePkg::decode_from(payload)?;
    let server_config = crate::state::get::get_server_config();
    // verify identy
    exchange_pkg.verify_identity(&server_config.identity)?;
    let src_endpoint = exchange_pkg.get_baseinfo();
    let dst_endpoint = match exchange_pkg.get_target() {
        Some(target) => target,
        None => {
            warn!("No target found in exchange package");
            return Ok(());
        }
    };
    info!("Exchange package: src: {:?}, dst: {:?}", src_endpoint, dst_endpoint);
    let udp_socket = crate::state::get::get_udp_socket();
    let encoded = BaseUp2pProtocol::pakge_exchange_with_payload(exchange_pkg)?.encode_to_vec()?;
    let lock = DEVICE_LIST.read().await;
    let exchange_endpoint  = lock.get(&dst_endpoint.get_global_id())
            .ok_or_else(|| anyhow::anyhow!(""))?.clone();
    drop(lock);
    udp_socket.send_to(&encoded, exchange_endpoint).await?;
    Ok(())
} 