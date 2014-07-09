use std::io::{TcpListener, Listener, Acceptor, TimedOut};

pub fn start_server() -> () {
    let listener = TcpListener::bind("127.0.0.1", 9021);
    let mut acceptor = listener.listen().unwrap();

    for stream in acceptor.incoming() {
        match stream {
            Err(e) => { /* TODO: handle errors */ }
            Ok(stream) => spawn(proc() {

            })
        }
    }

    drop(acceptor);
}
