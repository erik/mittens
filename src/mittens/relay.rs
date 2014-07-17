use std::io::{TcpListener, Listener, Acceptor};
use mittens::socks;
use mittens::config::Config;

pub fn start_server(conf: Config) {
    let listener = TcpListener::bind("127.0.0.1", 1080);
    let mut acceptor = listener.listen().unwrap();

    for stream in acceptor.incoming() {
        match stream {
            Err(_) => { /* TODO: handle errors */ }
            Ok(mut stream) => socks::handle_stream(&mut stream)
        }
    }

    drop(acceptor);
}
