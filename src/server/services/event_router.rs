use std::{collections::HashMap, net::SocketAddr, sync::{Arc, LazyLock}};

use tokio::sync::Mutex;
use tracing::{debug, info, warn};
use up2p::core::{bincodec::BinCodec, get_global_id::GetGlobalId, uprotocol_pkg::{ClientHelloPkg, ClientRequestAckPkg, ClientRequestPkg}, BaseUp2pProtocol};

use super::udp_event_handle::Up2pEvent;

static DEVICE_LIST: LazyLock<Arc<Mutex<HashMap<String, SocketAddr>>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

pub async fn route(event: Up2pEvent) {
    let ubase_protocal_pkg = event.get_data();
    let base_bind_result = BaseUp2pProtocol::decode_from(&ubase_protocal_pkg);
    match base_bind_result {
        Err(e) => {
            warn!("Failed to decode base protocal package: {:?}", e);
        },
        Ok(base_protocal) => {
            info!("Recieved a base protocal package");
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
                _ => {
                    warn!("Unkown base protocal type: {:?}", base_protocal);
                }
            }
        }
    }
}

async fn handle_client_hello_pkg(payload: &[u8], endpoint_addr: SocketAddr) -> anyhow::Result<()> {
    let _identitier = crate::state::get::get_identitier();
    if let Ok((clien_hello_pkg, _size)) = bincode::decode_from_slice::<
        ClientHelloPkg, bincode::config::Configuration
    >(payload, up2p::get_binencode_config()) {
        if !_identitier(&clien_hello_pkg.get_identity()) {
            info!("Conn Refused: {}", endpoint_addr);
            return Ok(());
        }
        match clien_hello_pkg.get_msg() {
            ClientHelloPkg::MSG_HELLO => {
                info!("Client hello: {}", endpoint_addr);
                // Add the device to the device list
                DEVICE_LIST.lock().await.insert(clien_hello_pkg.get_global_id(), endpoint_addr);
                let udp_socket = crate::state::get::get_udp_socket();
                let pp = BaseUp2pProtocol::hello_ack_with_payload()?;
                let encoded = pp.encode_to_vec()?;
                debug!("Encoded response: {:?}", encoded);
                udp_socket.send_to(&encoded, endpoint_addr).await?;
                debug!("Device list: {:?}", DEVICE_LIST.lock().await);
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
    let _identitier = crate::state::get::get_identitier();
    if let Ok((clien_hello_pkg, _size)) = bincode::decode_from_slice::<
        ClientRequestPkg, bincode::config::Configuration
    >(payload, up2p::get_binencode_config()) {
        if !_identitier(&clien_hello_pkg.get_identity()) {
            info!("Conn Refused: {}", endpoint_addr);
            return Ok(());
        }
        match clien_hello_pkg.get_request_type() {
            ClientRequestPkg::REQUEST_ENDPOINT => {
                info!("Client request endpoint: {}", endpoint_addr);
                // Add the device to the device list
                let requested_global_id = clien_hello_pkg.get_payload_as_global_id()?;
                if let Some(ov) = DEVICE_LIST.lock().await.get(&requested_global_id) {
                    let pp = BaseUp2pProtocol::response_with_payload(ClientRequestAckPkg::new(format!("{}:{}", ov.ip(), ov.port())))?;
                    let encoded = bincode::encode_to_vec(&pp, up2p::get_binencode_config())?;
                    udp_socket.send_to(&encoded, endpoint_addr).await?;
                } else {
                    warn!("Requested device not found: {}", requested_global_id);
                };
                debug!("Device list: {:?}", DEVICE_LIST.lock().await);
            },
            _=> warn!("Unkown client request type: {:?}", clien_hello_pkg)
        }
    } else {
        warn!("Failed to decode client request package, payload: {:?}", payload);
    }
    Ok(())
}