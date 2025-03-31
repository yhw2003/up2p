use std::any::Any;

use crate::core::{uprotocol_pkg::ClientRequestAckPkg, BaseUp2pProtocol};

pub trait CliEvent: Send + Any + 'static {
    fn get_event_type(&self) -> u8;
    fn as_any(&self) -> &dyn Any;
}

pub struct EventType;
impl EventType {
    pub const HELLO_ACK: u8 = BaseUp2pProtocol::TYPE_HELLO_ACK;
    pub const REQUEST_ACK: u8 = BaseUp2pProtocol::TYPE_REQUEST_ACK;
    pub const P2P_PKG_EXCHANGE: u8 = BaseUp2pProtocol::TYPE_PKG_EXCHANGE;
}

#[derive(Debug)]
pub struct HelloACKEvent;

impl CliEvent for HelloACKEvent {
    fn get_event_type(&self) -> u8 {
        EventType::HELLO_ACK as u8
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct RequestAckEvent {
    endpoint_address: String,
}

impl CliEvent for RequestAckEvent {
    fn get_event_type(&self) -> u8 {
        EventType::REQUEST_ACK as u8
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl RequestAckEvent {
    pub fn new(client_request_ack_pkg: ClientRequestAckPkg) -> Self {
        Self { endpoint_address: client_request_ack_pkg.get_endpoint_address() }
    }
    pub fn get_result_endpoint_address(&self) -> String {
        self.endpoint_address.clone()
    }
}

#[derive(Debug)]
pub struct PkgExchangeEvent {
    pub payload: Vec<u8>,
}

impl CliEvent for PkgExchangeEvent {
    fn get_event_type(&self) -> u8 {
        EventType::P2P_PKG_EXCHANGE as u8
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl PkgExchangeEvent {
    pub fn get_payload(&self) -> Vec<u8> {
        self.payload.clone()
    }
}