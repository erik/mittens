use std::io::net::ip::SocketAddr;

pub struct Config {
    server: ServerConfig,
    relay: RelayConfig
}

pub struct ServerConfig;

pub struct RelayConfig {
    pub server_addr: SocketAddr
}
