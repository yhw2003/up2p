use std::net::{IpAddr, SocketAddr};

pub fn get_global_id(client_class: &str, client_instance: &str) -> String {
    format!("{}-{}", client_class, client_instance)
}

pub fn parse_ip_port(input: &str) -> Option<(IpAddr, u16)> {
    input.parse::<SocketAddr>().ok().map(|addr| (addr.ip(), addr.port()))
}