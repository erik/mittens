extern crate serialize;
use self::serialize::base64::{ToBase64, STANDARD};

use std::io::{BufferedStream, IoError, TcpListener, Listener, Acceptor, TcpStream, IoResult};

use knuckle::sign::{SignedMsg, PublicKey};
use knuckle::util::random_bytes;

use mittens::socks;
use mittens::config::RelayConfig;

struct ServerConnection {
    server_pubkey: PublicKey,
    control: BufferedStream<TcpStream>
}

impl ServerConnection {
    fn new(conf: &RelayConfig) -> IoResult<ServerConnection> {
        let control = try!(TcpStream::connect_timeout(conf.server_addr, 5000));
        let mut buffered = BufferedStream::new(control);

        try!(ServerConnection::verify_stream(conf.server_pubkey, &mut buffered));

        Ok(ServerConnection {
            server_pubkey: conf.server_pubkey,
            control: buffered
        })
    }

    /// Challenge the server with a random nonce, ensure that they can
    /// produce a valid signature.
    fn verify_stream(key: PublicKey, stream: &mut BufferedStream<TcpStream>) -> IoResult<()> {
        // Generate bytes and base64 encode
        let nonce = random_bytes(128);
        let nonce_b64 = nonce.as_slice().to_base64(STANDARD);

        try!(stream.write_line(nonce_b64.as_slice()));
        let resp = try!(stream.read_line());

        let smsg = SignedMsg {
            pk: key,
            signed: Vec::from_slice(resp.as_bytes())
        };

        match smsg.verify() {
            Some(ref msg) if *msg == nonce => Ok(()),
            _ => Err(IoError::last_error())
        }
    }
}


pub fn start_relay(conf: RelayConfig) {
    let server_conn = match ServerConnection::new(&conf) {
        Ok(conn) => conn,
        Err(x) => fail!("Failed to establish secure connection: {}", x)
    };

    let listener = TcpListener::bind(conf.relay_host.as_slice(),
                                     conf.relay_port);
    let mut acceptor = listener.listen().unwrap();

    for stream in acceptor.incoming() {
        match stream {
            Err(_) => { /* TODO: handle errors */ }
            Ok(mut stream) => socks::handle_stream(&mut stream)
        }
    }

    drop(acceptor);
}
