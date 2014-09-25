use std::io::{IoError, TcpListener, Listener, Acceptor, TcpStream, IoResult};
use std::time::duration::Duration;

use knuckle::{cryptobox, sign};
use knuckle::util::random_bytes;
use knuckle::cryptobox::{CryptoBox, Keypair};

use mittens::socks;
use mittens::config::RelayConfig;

struct ServerConnection {
    verify_key: sign::PublicKey,
    cbox: cryptobox::CryptoBox,
    control: TcpStream
}

impl ServerConnection {
    fn new(conf: &RelayConfig) -> IoResult<ServerConnection> {
        let control = try!(TcpStream::connect_timeout(conf.server_addr,
                                                      Duration::seconds(5000)));

        // TODO: REMOVE ME
        let key = Keypair::new();

        Ok(ServerConnection {
            verify_key: conf.verify_key,
            cbox: CryptoBox::from_key_pair(key.sk, key.pk),
            control: control
        })
    }

    /// Challenge the server with a random nonce, ensure that they can
    /// produce a valid signature.
    fn stream_handshake(&mut self) -> IoResult<()> {
        let nonce = random_bytes(128);
        let resp = try!(self.send(nonce.as_slice()));

        let smsg = sign::SignedMsg {
            pk: self.verify_key,
            signed: resp
        };

        match smsg.verify() {
            Some(ref msg) if *msg == nonce => Ok(()),
            _ => Err(IoError::last_error())
        }
    }

    fn rotate_key(&mut self) -> IoResult<()> {
        let key = random_bytes(32);
        try!(self.send(key.as_slice()));

        Ok(())
    }

    fn establish_connection(&mut self) -> IoResult<()> {
        try!(self.stream_handshake());
        try!(self.rotate_key());

        Ok(())
    }

    fn send(&mut self, msg: &[u8]) -> IoResult<Vec<u8>> {
        let ciphertext = self.cbox.encrypt(msg).as_bytes();
        self.send_raw(ciphertext.as_slice())
    }

    fn send_raw(&mut self, msg: &[u8]) -> IoResult<Vec<u8>> {
        try!(self.control.write_be_uint(msg.len()));
        try!(self.control.write(msg.as_slice()));

        let len = try!(self.control.read_be_uint());
        self.control.read_exact(len)
    }
}


pub fn start_relay(conf: RelayConfig) {
    let _ = match ServerConnection::new(&conf)
        .and_then(|mut c| c.establish_connection()) {
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
