use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use super::{bincodec::BinCodec, uprotocol_pkg::{ClientHelloPkg, ClientRequestAckPkg, ClientRequestPkg, PeerExchangePkg}};// udp包最大大小

// 定义了这个app通信的基本协议
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct BaseUp2pProtocol {
    content_len: u8,
    package_type: u8,
    content: Vec<u8>,
}

impl BaseUp2pProtocol {
    // client hello typed pkgs
    pub const TYPE_HELLO: u8 = 0x01;
    pub const TYPE_HELLO_ACK: u8 = 0x02;
    pub const TYPE_DATA: u8 = 0x03;
    pub const TYPE_DATA_ACK: u8 = 0x04;
    pub const TYPE_REQUEST: u8 = 0x05;
    pub const TYPE_REQUEST_ACK: u8 = 0x06;
    pub const TYPE_PKG_EXCHANGE: u8 = 0x07;
    pub fn client_hello_with_payload(_payload: ClientHelloPkg) -> anyhow::Result<Self> {
        let payload = _payload.encode_to_vec()?;
        Ok(BaseUp2pProtocol {
            content_len: payload.len() as u8,
            package_type: Self::TYPE_HELLO,
            content: payload,
        })
    }
    pub fn request_with_payload(_payload: ClientRequestPkg) -> anyhow::Result<Self> {
        let payload = _payload.encode_to_vec()?;
        if payload.len() > u8::MAX as usize {
            return Err(anyhow::anyhow!("payload too large"));
        }
        Ok(BaseUp2pProtocol {
            content_len: payload.len() as u8,
            package_type: Self::TYPE_REQUEST,
            content: payload,
        })
    }
    pub fn response_with_payload(_payload: ClientRequestAckPkg) -> anyhow::Result<Self> {
        let payload = _payload.encode_to_vec()?;
        if payload.len() > u8::MAX as usize {
            return Err(anyhow::anyhow!("payload too large"));
        }
        Ok(BaseUp2pProtocol {
            content_len: payload.len() as u8,
            package_type: Self::TYPE_REQUEST_ACK,
            content: payload.to_vec(),
        })
    }
    pub fn hello_ack_with_payload() -> anyhow::Result<Self> {
        Ok(BaseUp2pProtocol {
            content_len: 0,
            package_type: Self::TYPE_HELLO_ACK,
            content: vec![],
        })
    }
    pub fn pakge_exchange_with_payload(_payload: PeerExchangePkg) -> anyhow::Result<Self> {
        let payload = _payload.encode_to_vec()?;
        Ok(BaseUp2pProtocol {
            content_len: payload.len() as u8,
            package_type: Self::TYPE_PKG_EXCHANGE,
            content: payload,
        })
    }
    pub fn get_pkg_type(&self) -> u8 {
        self.package_type
    }
    pub fn get_payload(&self) -> &[u8] {
        &self.content
    }
}

impl Default for BaseUp2pProtocol {
    fn default() -> Self {
        BaseUp2pProtocol {
            content_len: 0,
            package_type: Self::TYPE_HELLO,
            content: Vec::new(),
        }
    }
}
