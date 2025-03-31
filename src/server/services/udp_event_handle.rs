use std::net::SocketAddr;

use tokio::{sync::mpsc, task::JoinHandle};
use tracing::warn;

pub async fn udp_event_handle() -> anyhow::Result<(mpsc::Receiver<Up2pEvent>, JoinHandle<()>)>  {
    let udp_socket = crate::state::get::get_udp_socket();
    let (tx, rx) = mpsc::channel(32);
    let handle = tokio::spawn(async move {
        let mut udp_buf = vec![0u8; 1500];
        let mut udp_err_cnt = 0_u64;
        loop {
            if let Ok((size, addr)) = udp_socket.recv_from(&mut udp_buf).await {
                let data = udp_buf[..size].to_vec();
                let event = Up2pEvent::new(data, addr);
                if let Err(err) = tx.send(event).await {
                    warn!("Failed to send udp event to channel, {}", err);
                }
            } else {
                udp_err_cnt += 1;
                warn!("Failed to receive data from udp socket, total error count: {}", udp_err_cnt);
            };
        }
    });
    Ok((rx, handle))
}

pub struct Up2pEvent {
    data: Vec<u8>,
    addr: SocketAddr,
}

impl Up2pEvent {
    fn new(data: Vec<u8>, addr: SocketAddr) -> Self {
        Up2pEvent { data, addr }
    }
    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }
    pub fn get_addr(&self) -> SocketAddr {
        self.addr
    }
}