use bincode::{Decode, Encode};


#[derive(Debug, Clone, Encode, Decode)]
pub struct RequestInfo {
    pub client_class: String,
    pub client_instance: String
}