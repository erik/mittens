use knuckle::sign::{Keypair, PublicKey};

use std::io::net::ip::SocketAddr;

pub struct Config {
    server: ServerConfig,
    relay: RelayConfig
}

pub struct ServerConfig {
    signing_key: Keypair
}

pub struct RelayConfig {
    /// Bind host / port
    pub relay_host: String,
    pub relay_port: u16,

    /// Remote host / port
    pub server_addr: SocketAddr,
    /// Asserted public key for remote server handshake
    pub verify_key: PublicKey
}
