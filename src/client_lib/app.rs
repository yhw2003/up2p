use std::{cell::Cell, net::{IpAddr, SocketAddr}, pin::Pin, sync::Arc, time::Duration, u64};
use anyhow::anyhow;
use tokio::{net::UdpSocket, sync::{mpsc::{Receiver, Sender}, oneshot, Mutex}, task::JoinHandle};
use tracing::{debug, info, trace, warn};
use tracing_subscriber::field::debug;
use crate::{client_lib::event::PkgExchangeEvent, core::{bincodec::BinCodec, request_info::RequestInfo, uprotocol_pkg::{BasePkg, ClientHelloPkg, ClientRequestAckPkg, ClientRequestPkg, GetBaseInfo, PeerExchangePkg}, BaseUp2pProtocol}};

use super::event::{CliEvent, EventType, HelloACKEvent, RequestAckEvent};

pub struct Up2pCli {
    base_info: BasePkg,
    udp_socket: Arc<UdpSocket>,
    server_address: (IpAddr, u16),
    stop_sig: Option<tokio::sync::oneshot::Receiver<()>>,
    event_list: Arc<Mutex<Vec<(Sender<Option<Box<dyn CliEvent>>>, u8, u128)>>>,
    event_reciver: Cell<Option<Receiver<Box<dyn CliEvent>>>>,
    event_loop_handle: Option<JoinHandle<()>>,
    event_task: Cell<Option<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>>,
    exchange_buffer: Arc<Mutex<Vec<PkgExchangeEvent>>>,
}

unsafe impl Sync for Up2pCli {}

impl Up2pCli {
    /// let (up2p_cli, cancer_hdl) = Up2pCli::new(base_info, udp_socket, server_address); 
    pub fn new(base_info: BasePkg, udp_socket: Arc<UdpSocket>, server_address:(IpAddr, u16) ) -> (Self, oneshot::Sender<()>) {
        let (event_tx, event_rx) = tokio::sync::mpsc::channel(1024);
        let (cancel_tx, cancell_rx) = tokio::sync::oneshot::channel();
        let _udp_socket = udp_socket.clone();
        let event_task = Box::pin(async move {
            info!("event loop started");
            let mut buf = [0u8; 1500];
            loop {
                let (len, endpoint_addr) = match _udp_socket.recv_from(&mut buf).await {
                    Ok((len, endpoint_addr)) => (len, endpoint_addr),
                    Err(e) => {
                        warn!("recv_from error: {}", e);
                        continue;
                    }
                };
                trace!("recv_from: len: {}, endpoint_addr: {}", len, endpoint_addr);
                // handle udp pkg
                let base_protocol_pkg = match BaseUp2pProtocol::decode_from(&buf[..len])
                {
                    Ok(base_protocol_pkg) => base_protocol_pkg,
                    Err(e) => {
                        warn!("decode_from_slice error: {}", e);
                        continue;
                    }
                };
                let boxed_client_event: Box<dyn CliEvent> = match base_protocol_pkg.get_pkg_type() {
                    BaseUp2pProtocol::TYPE_PKG_EXCHANGE => {
                        let payload = base_protocol_pkg.get_payload();
                        let peer_exchange_pkg = match PeerExchangePkg::decode_from(payload) {
                            Ok(peer_exchange_pkg) => peer_exchange_pkg,
                            Err(e) => {
                                warn!("decode_from_slice error: {}", e);
                                continue;
                            }
                        };
                        Box::new(PkgExchangeEvent::new(
                            peer_exchange_pkg.get_payload(),
                            peer_exchange_pkg.get_baseinfo().clone(),
                            peer_exchange_pkg.get_target()
                        )) as Box<dyn CliEvent>
                    }
                    BaseUp2pProtocol::TYPE_HELLO_ACK => {
                        Box::new(HelloACKEvent) as Box<dyn CliEvent>
                    }
                    BaseUp2pProtocol::TYPE_REQUEST_ACK => {
                        let payload = base_protocol_pkg.get_payload();
                        let client_request_ack_pkg = match ClientRequestAckPkg::decode_from(payload) {
                            Ok(client_request_ack_pkg) => client_request_ack_pkg,
                            Err(e) => {
                                warn!("decode_from_slice error: {}", e);
                                continue;
                            }
                        };
                        Box::new(RequestAckEvent::new(client_request_ack_pkg)) as Box<dyn CliEvent>
                    }
                    _ => {
                        warn!("unknown pkg type: {}", base_protocol_pkg.get_pkg_type());
                        continue;
                    }
                };
                // send event to event loop
                match event_tx.send(boxed_client_event).await {
                    Ok(_) => {
                        trace!("send event to event loop");
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
            event_list: Arc::new(Mutex::new(Vec::new())),
            event_reciver: Cell::new(Some(event_rx)),
            event_loop_handle: None,
            event_task: Cell::new(Some(event_task)),
            stop_sig: Some(cancell_rx),
            exchange_buffer: Arc::new(Mutex::new(Vec::new())),
        }, cancel_tx)
    }
    pub async fn start(&self) -> anyhow::Result<()> {
        let mut event_reciver = self.event_reciver.take().expect("client has been started");
        let event_task = self.event_task.take().expect("event loop has been started");
        tokio::spawn(async move { event_task.await });
        let event_list = self.event_list.clone();
        let aeb = self.exchange_buffer.clone();
        tokio::spawn(async move {
            debug!("start to handle event loop");
            loop {
                match event_reciver.recv().await {
                    Some(event) => {
                        let recived_event_type = event.get_event_type();
                        match recived_event_type {
                            EventType::P2P_PKG_EXCHANGE => {
                                let pkg_exchange_event = match event.as_any().downcast_ref::<PkgExchangeEvent>() {
                                    Some(pkg_exchange_event) => pkg_exchange_event.clone(),
                                    None => {
                                        warn!("downcast error for PkgExchangeEvent");
                                        continue;
                                    }
                                };
                                // todo!
                                // 我没想明白为社么这个锁要在这里 如果在```if !ok_flag```那一行上锁它就会死锁
                                /*  分析得到的关键日志：
                                 *  DEBUG up2p::client_lib::app: get lock for exchange buffer(recv)
                                 *  DEBUG up2p::client_lib::app: eb lock dropped, waiting for event
                                 *  DEBUG up2p::client_lib::app: writing to exchange buffer
                                 *  DEBUG up2p::client_lib::app: event loop recv event
                                 *  DEBUG up2p::client_lib::app: scanning, finde event type: 7, recived event type: 7
                                 *  DEBUG up2p::client_lib::app: event loop recv event
                                 *  DEBUG up2p::client_lib::app: scanning, finde event type: 7, recived event type: 7
                                 *  DEBUG up2p::client_lib::app: event loop recv event
                                 *  DEBUG up2p::client_lib::app: scanning, finde event type: 7, recived event type: 7
                                 *  DEBUG up2p::client_lib::app: event received with payload: true
                                 */
                                let mut eb = aeb.lock().await;
                                debug!("ev lenth: {}", eb.len());
                                let event_list = event_list.lock().await;
                                let mut ok_flag = false;
                                for (event_tx, event_type, _) in event_list.iter() {
                                    debug!("scanning, finde event type: {}, recived event type: {}", event_type, recived_event_type);
                                    if *event_type == recived_event_type {
                                        ok_flag = true;
                                        if let Err(e) = event_tx.send(Some(Box::new(pkg_exchange_event.clone()) as Box<dyn CliEvent>)).await {
                                            warn!("send event error: {}", e);
                                        };
                                        break;
                                    }
                                }
                                drop(event_list);
                                if !ok_flag {
                                    // let mut eb = aeb.lock().await;
                                    debug!("writing to exchange buffer");
                                    eb.push(pkg_exchange_event);
                                }
                            }
                            EventType::HELLO_ACK => {
                                let mut event_list = event_list.lock().await;
                                for (idx, (event_tx, event_type, _)) in event_list.iter().enumerate() {
                                    debug!("scanning, finde event type: {}, recived event type: {}", event_type, recived_event_type);
                                    if *event_type == recived_event_type {
                                        if let Err(e) = event_tx.send(None).await {
                                            warn!("send event error: {}", e);
                                        };
                                        event_list.remove(idx);
                                        break;
                                    }
                                }
                                drop(event_list);
                            }
                            EventType::REQUEST_ACK => {
                                let event_list = event_list.lock().await;
                                for (event_tx, event_type, _) in event_list.iter() {
                                    debug!("scanning, finde event type: {}, recived event type: {}", event_type, recived_event_type);
                                    if *event_type == recived_event_type {
                                        if let Err(e) = event_tx.send(Some(event)).await {
                                            warn!("send event error: {}", e);
                                        };
                                        break;
                                    }
                                }
                                drop(event_list);
                            }
                            _ => {
                                warn!("unknown event type: {}", recived_event_type);
                            }
                            
                        }
                    }
                    None => {
                        warn!("event receiver closed");
                    }
                }
                debug!("event loop recv event");
            }
        });
        self.event_reciver.set(None);
        Ok(())
    }
    // send client hello to server
    pub async fn client_hello(&self) -> anyhow::Result<()> {
        let hello_pkg = BaseUp2pProtocol::client_hello_with_payload(
            ClientHelloPkg::new(&self.base_info.client_class, &self.base_info.client_instance, &self.base_info.identity, 0x01)
        )?;
        self.udp_socket.send_to(hello_pkg.encode_to_vec()?.as_slice(), self.server_address).await?;
        // wait for response
        self.subscribe_ack_event(EventType::HELLO_ACK, Duration::from_secs(3)).await?;
        Ok(())
    }
    // send client request to server
    pub async fn client_request(&self, _req: RequestInfo) -> anyhow::Result<Option<String>> {
        let req = ClientRequestPkg::create_endpoint_request(
            &self.base_info.client_class,
            &self.base_info.client_instance,
            &self.base_info.identity,
            &crate::utils::get_global_id(&_req.client_class, &_req.client_instance)
        );
        let request_pkg = BaseUp2pProtocol::request_with_payload(
            req
        )?;
        self.udp_socket.send_to(request_pkg.encode_to_vec()?.as_slice(), self.server_address).await?;
        // wait for response
        let response = self.subscribe_ack_event(EventType::REQUEST_ACK, Duration::from_secs(3)).await?;
        if let Some(event) = response {
            if let Some(request_ack) = event.as_any().downcast_ref::<RequestAckEvent>() {
                debug!("request ack: {:?}", request_ack);
                return Ok(Some(request_ack.get_result_endpoint_address()));
            } else {
                warn!("unknown event type: {}", event.get_event_type());
            }
        } else {
            warn!("request ack not received");
        }
        // maybe type error
        Err(anyhow!("request ack type mismatch"))
    }
    // subscribe ack event
    // Ok(None) if event is received bug no payload
    // Err(anyhow::anyhow!("event timeout")) if event is not received
    // Ok(Some(event)) if event is received with payload
    async fn subscribe_ack_event(&self, event_type: u8, cancel_duration: Duration) -> anyhow::Result<Option<Box<dyn CliEvent>>> {
        let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(1);
        let id = rand::random::<u128>();
        let mut event_list = self.event_list.lock().await;
        debug!("get event_list lock");
        event_list.push((event_tx, event_type, id));
        drop(event_list);
        debug!("event_list lock dropped");
        let wait_result: Option<Box<dyn CliEvent>> = tokio::select! {
            result = event_rx.recv() => {
                match result {
                    Some(event) => {
                        debug!("event received with payload: {:?}", event.is_some());
                        event
                    }
                    None => {
                        warn!("event receiver closed");
                        None
                    }
                }
            }
            _ = tokio::time::sleep(cancel_duration) => {
                warn!("event timeout: {}, event id dropped: {}", event_type, id);
                return Err(anyhow::anyhow!("event timeout"));
            }
        };
        let mut event_list = self.event_list.lock().await;
        event_list.retain(|(_, _, event_id)| *event_id != id);
        Ok(wait_result)
    }


    // communicate witch other peer
    // you can also send pkg to server, the server will forward it to other peer
    pub async fn pkg_send_to(&self, endpoint_addr: SocketAddr, payload: Vec<u8>, target: Option<BasePkg>) -> anyhow::Result<()> {
        let pkg = BaseUp2pProtocol::pakge_exchange_with_payload(
            PeerExchangePkg::new(
                self.base_info.clone(),
                payload,
                target
            )
        )?;
        self.udp_socket.send_to(pkg.encode_to_vec()?.as_slice(), endpoint_addr).await?;
        Ok(())
    }

    pub async fn pkg_recv_from(&self) -> anyhow::Result<(BasePkg, Vec<u8>)> {

        // let ret = self.subscribe_ack_event(
        //     EventType::P2P_PKG_EXCHANGE, Duration::from_secs(u64::MAX)
        // ).await?.expect("event is None");
        // let ret  = ret.as_any().downcast_ref::<PkgExchangeEvent>().unwrap();
        let ret = {
            let mut eb = self.exchange_buffer.lock().await;
            debug!("get lock for exchange buffer(recv)");
            if eb.len() == 0 {
                drop(eb);
                debug!("eb lock dropped, waiting for event");
                let ret = self.subscribe_ack_event(
                    EventType::P2P_PKG_EXCHANGE, 
                Duration::from_secs(u64::MAX)
                ).await?.expect("event is None");
                let r = ret.as_any().downcast_ref::<PkgExchangeEvent>().unwrap().clone();
                debug!("event received {:?}", r);
                r
            } else {
                debug!("reading from exchange buffer");
                let r = eb.remove(0);
                drop(eb);
                debug!("eb lock dropped");
                r
            }
        };
        if let Some(dst) = ret.get_dst() {
            if dst != self.base_info {
                warn!("Received pkg not for self");
                return Err(anyhow!("pkf not for self"));
            }
        }
        let data = ret.get_payload();
        return Ok((ret.get_src(), data));
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