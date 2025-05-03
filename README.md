# up2p
这是一个rust实现的p2p框架，它的主要目标在于跨平台和简单易用

！！这个crate是一个玩具的一部分，它现在完全不生产可用，或许webrtc会是一个更好的选项，你可以再todo列表中看到它目前的问题。

！！传输内容完全明文，身份校验并不完整，请不要传输任何保密内容。

！！当打洞无法成功时，服务器转发效率并不高

----

### 使用案例
``` rust
// main.rs
async fn main () -> anyhow::Result<()> {
    let client_config = ClientConfig::parse_config().unwrap();
    tracing_subscriber::fmt().with_max_level(Level::from_str(&client_config.log_level)?).init();
    let udp_socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await.unwrap());
    let server_address = client_config.server_address.parse::<std::net::SocketAddr>().unwrap();
    let up2p_client = Up2pCli::new(BasePkg {
        client_instance: client_config.client_instance,
        client_class: "cli".to_string(),
        identity: client_config.identity,
    }, udp_socket, (server_address.ip(), server_address.port()));
    up2p_client.0.start().await.unwrap();
    up2p_client.0.client_hello().await?;
    info!("client hello down");
    loop {
        let r = up2p_client.0.pkg_recv_from().await?;
        info!("recv pkg: {:?}", r);
    }
}
```



### todo
- 基础功能（再这些功能实现前，我都不认为它是生产可用的）
    1. 支持tls，更严格的身份验证
    1. 充分的单元测试
    1. 更多example和完善的文档
    1. 支持类型安全的传输方式，而不是发送Vec\<u8\>
    1. 支持打开stream，不止于发送简单报文
    1. 可感知的丢包