//! SOCKSv5 protocol support

use std::io::{IoError, IoResult, Reader, TcpStream};

static VERSION_NUMBER: u8 = 0x05;
static AUTH_NONE: u8 = 0x00;
static CONNECT_CMD: u8 = 0x01;

struct SocksConnection<'a> {
    stream: &'a mut TcpStream
}

impl <'a> SocksConnection<'a> {
    /// Make sure that the client sends the right version number at
    /// the start of the message, or fail
    fn do_client_version(&mut self) -> IoResult<()> {
        match try!(self.stream.read_byte()) {
            VERSION_NUMBER => Ok(()),
            _ => Err(IoError::last_error())
        }
    }

    /// Client -> Server:
    ///
    /// |VER | NMETHODS | METHODS  |
    /// |----+----------+----------|
    /// | 1  |    1     | 1 to 255 |
    ///
    /// Client <- Server:
    /// +----+--------+
    /// |VER | METHOD |
    /// +----+--------+
    /// | 1  |   1    |
    fn do_client_hello(&mut self) -> IoResult<()> {
        try!(self.do_client_version());

        // Number of authentication methods
        let nmethods = try!(self.stream.read_byte());

        // Authentication methods
        let methods = try!(self.stream.read_exact(nmethods as uint));

        // If the client doesn't respond to NULL-auth, we can't proceed.
        if !methods.contains(&AUTH_NONE) {
            // Send the error back
            try!(self.stream.write(&[VERSION_NUMBER, 0xFF]));

            return Err(IoError::last_error());
        }

        // Now respond
        try!(self.stream.write(&[VERSION_NUMBER, AUTH_NONE]));

        Ok(())
    }

    /// Client -> Server:
    /// +----+-----+-------+------+----------+----------+
    /// |VER | CMD |  RSV  | ATYP | DST.ADDR | DST.PORT |
    /// +----+-----+-------+------+----------+----------+
    /// | 1  |  1  | X'00' |  1   | Variable |    2     |
    ///
    /// o  VER    protocol version: X'05'
    /// o  CMD
    ///    o  CONNECT X'01'
    ///    o  BIND X'02'
    ///    o  UDP ASSOCIATE X'03'
    /// o  RSV    RESERVED
    /// o  ATYP   address type of following address
    ///    o  IP V4 address: X'01'
    ///    o  DOMAINNAME: X'03'
    ///    o  IP V6 address: X'04'
    /// o  DST.ADDR desired destination address
    /// o  DST.PORT desired destination port in network octet
    ///    order
    ///
    /// Client <- Server:
    /// +----+-----+-------+------+----------+----------+
    /// |VER | REP |  RSV  | ATYP | BND.ADDR | BND.PORT |
    /// +----+-----+-------+------+----------+----------+
    /// | 1  |  1  | X'00' |  1   | Variable |    2     |
    ///
    ///   o  VER    protocol version: X'05'
    ///   o  REP    Reply field:
    ///      o  X'00' succeeded
    ///      o  X'01' general SOCKS server failure
    ///      o  X'02' connection not allowed by ruleset
    ///      o  X'03' Network unreachable
    ///      o  X'04' Host unreachable
    ///      o  X'05' Connection refused
    ///      o  X'06' TTL expired
    ///      o  X'07' Command not supported
    ///      o  X'08' Address type not supported
    ///      o  X'09' to X'FF' unassigned
    ///   o  RSV    RESERVED
    ///   o  ATYP   address type of following address
    ///
    ///      o  IP V4 address: X'01'
    ///      o  DOMAINNAME: X'03'
    ///      o  IP V6 address: X'04'
    ///   o  BND.ADDR       server bound address
    ///   o  BND.PORT       server bound port in network octet order
    fn do_client_request(&mut self) -> IoResult<()> {
        try!(self.do_client_version());

        // Commands (for now only connect is supported)
        match try!(self.stream.read_byte()) {
            CONNECT_CMD => (),
            _ => {
                // Tell client we don't support that command
                try!(self.stream.write(&[VERSION_NUMBER, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00]));
                return Err(IoError::last_error());
            }
        };

        // Reserved byte
        match try!(self.stream.read_byte()) {
            0u8 => (),
            _ => { return Err(IoError::last_error()); }
        };


        Ok(())
    }
}

pub fn handle_stream<'a>(stream: &'a mut TcpStream) -> () {
    let mut conn = SocksConnection { stream: stream };

    if conn.do_client_hello().is_err() {
        return;
    }

    loop {
        conn.do_client_request();
    }
}