use std::{cell::RefCell, net::IpAddr, pin::Pin, sync::Arc, time::{Duration, SystemTime}};
use bincode::config::Configuration;
use tokio::{net::UdpSocket, sync::{mpsc::Receiver, oneshot::Sender}, task::JoinHandle};
use tracing::{info, warn};
use crate::core::{bincodec::BinCodec, uprotocol_pkg::{BasePkg, GetBaseInfo}, BaseUp2pProtocol};

use super::event::{CliEvent, EventType, HelloACKStruct};

pub struct Up2pCli {
    base_info: BasePkg,
    udp_socket: Arc<UdpSocket>,
    server_address: (IpAddr, u16),
    stop_sig: Option<tokio::sync::oneshot::Receiver<()>>,
    event_list: Vec<(Sender<()>, u8, u128)>,
    event_reciver: Option<RefCell<Receiver<Box<dyn CliEvent>>>>,
    event_loop_handle: Option<JoinHandle<()>>,
    event_task: Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
}

impl Up2pCli {
    /// let (up2p_cli, cancer_hdl) = Up2pCli::new(base_info, udp_socket, server_address); 
    pub fn new(base_info: BasePkg, udp_socket: Arc<UdpSocket>, server_address:(IpAddr, u16) ) -> (Self, Sender<()>) {
        let (event_tx, event_rx) = tokio::sync::mpsc::channel(1024);
        let (cancel_tx, cancell_rx) = tokio::sync::oneshot::channel();
        let _udp_socket = udp_socket.clone();
        let event_task = Box::pin(async move {
            let mut buf: Vec<u8> = Vec::with_capacity(0);
            loop {
                let (len, endpoint_addr) = match _udp_socket.recv_from(&mut buf).await {
                    Ok((len, endpoint_addr)) => (len, endpoint_addr),
                    Err(e) => {
                        warn!("recv_from error: {}", e);
                        continue;
                    }
                };
                // handle udp pkg
                let (base_protocol_pkg, _) = match bincode::decode_from_slice::<BaseUp2pProtocol, Configuration>(&buf[..len], crate::get_binencode_config())
                {
                    Ok((base_protocol_pkg, size)) => (base_protocol_pkg, size),
                    Err(e) => {
                        warn!("decode_from_slice error: {}", e);
                        continue;
                    }
                };
                let boxed_client_event: Box<dyn CliEvent> = match base_protocol_pkg.get_pkg_type() {
                    BaseUp2pProtocol::TYPE_HELLO_ACK => {
                        Box::new(HelloACKStruct) as Box<dyn CliEvent>
                    }
                    _ => {
                        warn!("unknown pkg type: {}", base_protocol_pkg.get_pkg_type());
                        continue;
                    }
                };
                // send event to event loop
                match event_tx.send(boxed_client_event).await {
                    Ok(_) => {
                        info!("send event to event loop");
                    }
                    Err(e) => {
                        warn!("send event to event loop error: {}", e);
                    }
                }
            }
        });
        (Up2pCli {
            base_info,
            udp_socket,
            server_address,
            event_list: Vec::new(),
            event_reciver: Some(RefCell::new(event_rx)),
            event_loop_handle: None,
            event_task,
            stop_sig: Some(cancell_rx),
        }, cancel_tx)
    }
    pub async fn start(&mut self) -> anyhow::Result<()> {
        let mut event_reciver = unsafe { self.event_reciver.take().expect("this client has been running").as_ptr().read() };
        tokio::spawn(async move {
            loop {
                match event_reciver.recv().await {
                    Some(event) => {
                        let event_type = event.get_event_type();
                    }
                    None => {
                        warn!("event receiver closed");
                    }
                }
            }
        });
        self.event_task.as_mut().await;
        Ok(())
    }
    // send client hello to server
    pub async fn client_hello(&mut self) -> anyhow::Result<()> {
        let hello_pkg = BaseUp2pProtocol::client_hello_with_payload(&[])?;
        self.udp_socket.send_to(hello_pkg.encode_to_vec()?.as_slice(), self.server_address).await?;
        // wait for response
        self.subscribe_ack_event(EventType::HelloAck as u8, Duration::from_secs(1)).await?;
        Ok(())
    }

    // subscribe ack event
    async fn subscribe_ack_event(&mut self, event_type: u8, cancel_duration: Duration) -> anyhow::Result<()> {
        let (event_tx, event_rx) = tokio::sync::oneshot::channel();
        let id = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_millis();
        self.event_list.push((event_tx, event_type, id));
        let wait_result = tokio::select! {
            _ = event_rx => {
                Ok(())
            }
            _ = tokio::time::sleep(cancel_duration) => {
                warn!("event timeout: {}, event id dropped: {}", event_type, id);
                Err(anyhow::anyhow!("timeout"))
            }
        };
        self.event_list.retain(|(_, _, event_id)| *event_id != id);
        wait_result
    }

}


impl GetBaseInfo for Up2pCli {
    fn get_baseinfo(&self) -> &BasePkg {
        &self.base_info
    }
}

async fn handle_udp_pkg() {
    unimplemented!()
}