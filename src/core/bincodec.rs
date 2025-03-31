use bincode::{config::Configuration, Decode, Encode};

pub trait BinCodec<T> {
    fn decode_from(data: &[u8]) -> anyhow::Result<T>;
    fn encode_to_vec(&self) -> anyhow::Result<Vec<u8>>;
}

impl<T> BinCodec<T> for T
where
T: Encode + Decode<()>
{
    fn encode_to_vec(&self) -> anyhow::Result<Vec<u8>> {
        let s = bincode::encode_to_vec(self, crate::get_binencode_config())?;
        Ok(s)
    }

    fn decode_from(data: &[u8]) -> anyhow::Result<T> {
        let s = bincode::decode_from_slice::<Self, Configuration>(data, crate::get_binencode_config())?;
        Ok(s.0)
    }
}

#[cfg(test)]
mod test {
    use crate::core::{bincodec::BinCodec as _, uprotocol_pkg::ClientHelloPkg, BaseUp2pProtocol};

    #[tokio::test]
    async fn test_bin_codec() {
        let base_protocol_pkg = BaseUp2pProtocol::client_hello_with_payload(
            ClientHelloPkg::new("client_class", "client_instance", "identity", 0x00)
        ).unwrap();
        let data = base_protocol_pkg.encode_to_vec().unwrap();
        let base_protocol_pkg2 = BaseUp2pProtocol::decode_from(&data).unwrap();
        println!("base_protocol_pkg: {:?} encode -> {:?}", base_protocol_pkg, data);
        assert_eq!(base_protocol_pkg.get_pkg_type(), base_protocol_pkg2.get_pkg_type());
        assert_eq!(base_protocol_pkg.get_payload(), base_protocol_pkg2.get_payload());
    }
}