use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

pub trait PkgVerifyIdentity {
    fn verify_identity(&self, identity: &str) -> anyhow::Result<()>;
}

impl<T> PkgVerifyIdentity for T
where
    T: GetBaseInfo,
{
    fn verify_identity(&self, identity: &str) -> anyhow::Result<()> {
        if self.get_baseinfo().identity != identity {
            return Err(anyhow::anyhow!("Identity not match"));
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode, Clone)]
pub struct BasePkg {
    pub client_class: String,
    pub client_instance: String,
    pub identity: String,
}

// This trait means that you can get the base info from this struct
pub trait GetBaseInfo {
    fn get_baseinfo(&self) -> &BasePkg;
}

impl PartialEq for BasePkg {
    fn eq(&self, other: &Self) -> bool {
        self.client_class == other.client_class
            && self.client_instance == other.client_instance
    }
}

// client hello package
#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub struct ClientHelloPkg {
    baseinfo: BasePkg,
    msg: u8,
}

impl ClientHelloPkg {
    pub const MSG_HELLO: u8 = 0x01;
    pub const MSG_HEARTBEAT: u8 = 0x02;
    pub const MSG_LOGOUT: u8 = 0x03;
    pub const MSG_UPDATE: u8 = 0x04;
    pub fn new(client_class: &str, client_instance: &str, identity: &str, msg: u8) -> Self {
        Self {
            baseinfo: BasePkg {
                client_class: client_class.to_string(),
                client_instance: client_instance.to_string(),
                identity: identity.to_string(),
            },
            msg,
        }
    }
    pub fn get_msg(&self) -> u8 {
        self.msg
    }
    pub fn get_identity(&self) -> String {
        self.baseinfo.identity.clone()
    }
}

impl GetBaseInfo for ClientHelloPkg {
    fn get_baseinfo(&self) -> &BasePkg {
        &self.baseinfo
    }
    
}


// client request package
#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub struct ClientRequestPkg {
    baseinfo: BasePkg,
    request_type: u8,
    request_id: u8,
    request_payload: Vec<u8>,
}

impl ClientRequestPkg {
    pub const REQUEST_ENDPOINT: u8 = 0x01;
    pub const REQUEST_STATUS: u8 = 0x02;
    pub fn create_endpoint_request(client_class: &str, client_instance: &str, identity: &str, payload: &str) -> Self {
        Self {
            baseinfo: BasePkg {
                client_class: client_class.to_string(),
                client_instance: client_instance.to_string(),
                identity: identity.to_string()
            },
            request_type: Self::REQUEST_ENDPOINT,
            request_id: 0,
            request_payload: payload.as_bytes().to_vec(),
        }
    }
    pub fn get_request_type(&self) -> u8 {
        self.request_type
    }
    pub fn get_payload_as_global_id(&self) -> anyhow::Result<String> {
        Ok(String::from_utf8(self.request_payload.clone())?)
    }
    pub fn get_identity(&self) -> String {
        self.baseinfo.identity.clone()
    }
}

impl GetBaseInfo for ClientRequestPkg {
    fn get_baseinfo(&self) -> &BasePkg {
        &self.baseinfo
    }
    
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub struct ClientRequestAckPkg {
    endoint_address: String,
}

impl ClientRequestAckPkg {
    pub fn new(endoint_address: String) -> Self {
        Self {
            endoint_address,
        }
    }
    pub fn get_endpoint_address(&self) -> String {
        self.endoint_address.clone()
    }
}

// target required for pkg forward
#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub struct PeerExchangePkg {
    base_info: BasePkg,
    payload: Vec<u8>,
    target: Option<BasePkg>,
}

impl PeerExchangePkg {
    pub fn new(base_info: BasePkg, payload: Vec<u8>, target: Option<BasePkg>) -> Self {
        Self {
            base_info,
            payload,
            target
        }
    }
    pub fn get_payload(&self) -> Vec<u8> {
        self.payload.clone()
    }
    pub fn get_target(&self) -> Option<BasePkg> {
        self.target.clone()
    }
}

impl GetBaseInfo for PeerExchangePkg {
    fn get_baseinfo(&self) -> &BasePkg {
        &self.base_info
    }
}

#[cfg(test)]
mod test {
    use bincode::{config, Decode, Encode};
    use serde::{Deserialize, Serialize};

    use crate::core::uprotocol_pkg::BasePkg;

    #[derive(Debug, Serialize, Deserialize, Encode, Decode)]
    struct ReClientHelloPkg {
        client_class: String,
        client_instance: String,
        identity: String,
        msg: u8,
    }

    #[tokio::test]
    async fn test_serde_migrate() {
        // bin code of ReClientHelloPkg
        let re_client_hello_pkg = ReClientHelloPkg {
            client_class: "test".to_string(),
            client_instance: "test".to_string(),
            identity: "test".to_string(),
            msg: 0x01,
        };
        let encoded: Vec<u8> = bincode::encode_to_vec(&re_client_hello_pkg, config::standard()).unwrap();
        println!("encoded: {:?}", encoded);
        // bin code of ClientHelloPkg
        let client_hello_pkg = super::ClientHelloPkg::new("test", "test", "test", 0x01);
        let encoded2: Vec<u8> = bincode::encode_to_vec(&client_hello_pkg, config::standard()).unwrap();
        println!("encoded2: {:?}", encoded2);
        assert!(encoded == encoded2);
    }

    #[tokio::test]
    async fn test_base_info_eq() {
        let base_info1 = BasePkg {
            client_class: "test".to_string(),
            client_instance: "test".to_string(),
            identity: "test".to_string(),
        };
        let base_info2 = BasePkg {
            client_class: "test".to_string(),
            client_instance: "test".to_string(),
            identity: "test".to_string(),
        };
        assert_eq!(base_info1, base_info2);
        let base_info3 = BasePkg {
            client_class: "test".to_string(),
            client_instance: "test".to_string(),
            identity: "test_a".to_string(),
        };
        assert_eq!(base_info1, base_info3);
        let base_info4 = BasePkg {
            client_class: "test_a".to_string(),
            client_instance: "test".to_string(),
            identity: "test".to_string(),
        };
        assert_ne!(base_info1, base_info4);
        let base_info5 = BasePkg {
            client_class: "test".to_string(),
            client_instance: "test_a".to_string(),
            identity: "test".to_string(),
        };
        assert_ne!(base_info1, base_info5);
    }
}