//! SOCKSv5 protocol support

use std::str;
use std::io::{IoError, IoResult, Reader, TcpStream};
use std::io::net::ip::{Ipv4Addr, Ipv6Addr};
use std::io::net::addrinfo;

#[allow(dead_code)]
mod consts {
    pub static VERSION_NUMBER: u8 = 0x05;

    pub mod auth {
        pub static NONE: u8 = 0x00;
    }

    pub mod atype {
        pub static IPV4:   u8 = 0x1;
        pub static IPV6:   u8 = 0x4;
        pub static DOMAIN: u8 = 0x3;
    }

    pub mod command {
        pub static CONNECT:       u8 = 0x01;
        pub static BIND:          u8 = 0x02;
        pub static UDP_ASSOCIATE: u8 = 0x04;
    }

    pub mod reply {
        pub static SUCCESS:          u8 = 0x00;
        pub static GENERAL_FAILURE:  u8 = 0x01;
        pub static CONN_DENIED:      u8 = 0x02;
        pub static NET_UNREACHABLE:  u8 = 0x03;
        pub static HOST_UNREACHABLE: u8 = 0x04;
        pub static CONN_REFUSED:     u8 = 0x05;
        pub static TTL_EXPIRED:      u8 = 0x06;
        pub static CMD_UNSUPPORTED:  u8 = 0x07;
        pub static ADDR_UNSUPPORTED: u8 = 0x08;
    }
}

struct SocksConnection<'a> {
    stream: &'a mut TcpStream
}

impl <'a> SocksConnection<'a> {
    /// Make sure that the client sends the right version number at
    /// the start of the message, or fail
    fn read_client_version(&mut self) -> IoResult<()> {
        match try!(self.stream.read_byte()) {
            consts::VERSION_NUMBER => Ok(()),
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
    fn read_client_hello(&mut self) -> IoResult<()> {
        try!(self.read_client_version());

        // Number of authentication methods
        let nmethods = try!(self.stream.read_byte());

        // Authentication methods
        let methods = try!(self.stream.read_exact(nmethods as uint));

        // If the client doesn't respond to NULL-auth, we can't proceed.
        if !methods.contains(&consts::auth::NONE) {
            // Send the error back
            try!(self.stream.write(&[consts::VERSION_NUMBER, 0xFF]));

            return Err(IoError::last_error());
        }

        // Now respond that we're using NULL auth.
        try!(self.stream.write(&[consts::VERSION_NUMBER, consts::auth::NONE]));

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
    /// o  RSV    RESERVED
    /// o  ATYP   address type of following address
    /// o  DST.ADDR desired destination address
    /// o  DST.PORT desired destination port in network octet
    ///    order
    fn read_client_request(&mut self) -> IoResult<()> {
        try!(self.read_client_version());

        // Commands (for now only connect is supported)
        match try!(self.stream.read_byte()) {
            consts::command::CONNECT => (),
            _ => {
                // Tell client we don't support that command
                try!(self.send_error_reply(consts::reply::CMD_UNSUPPORTED));
                return Err(IoError::last_error());
            }
        };

        // Reserved byte
        match try!(self.stream.read_byte()) {
            0u8 => (),
            _ => { return Err(IoError::last_error()); }
        };

        // Address type
        let _ = match try!(self.stream.read_byte()) {
            // 4 bytes
            consts::atype::IPV4 => {
                let a = try!(self.stream.read_byte());
                let b = try!(self.stream.read_byte());
                let c = try!(self.stream.read_byte());
                let d = try!(self.stream.read_byte());

                Ipv4Addr(a, b, c, d)
            },

            // 16 bytes
            consts::atype::IPV6 => {
                let a = try!(self.stream.read_be_u16());
                let b = try!(self.stream.read_be_u16());
                let c = try!(self.stream.read_be_u16());
                let d = try!(self.stream.read_be_u16());
                let e = try!(self.stream.read_be_u16());
                let f = try!(self.stream.read_be_u16());
                let g = try!(self.stream.read_be_u16());
                let h = try!(self.stream.read_be_u16());

                Ipv6Addr(a, b, c, d, e, f, g, h)
            },

            // len | hostname
            consts::atype::DOMAIN => {
                let len = try!(self.stream.read_byte());
                let hostname = try!(self.stream.read_exact(len as uint));

                let mut hosts = match str::from_utf8(hostname.as_slice()) {
                    Some(host) => try!(addrinfo::get_host_addresses(host)),
                    None => { return Err(IoError::last_error()); }
                };

                match hosts.shift() {
                    Some(host) => host,
                    None => { return Err(IoError::last_error()); }
                }
            },

            // Unsupported address type
            _ => {
                try!(self.send_error_reply(consts::reply::ADDR_UNSUPPORTED));
                return Err(IoError::last_error());
            }
        };

        // Network port
        let _ = try!(self.stream.read_be_u16());

        Ok(())
    }

    /// Client <- Server:
    /// +----+-----+-------+------+----------+----------+
    /// |VER | REP |  RSV  | ATYP | BND.ADDR | BND.PORT |
    /// +----+-----+-------+------+----------+----------+
    /// | 1  |  1  | X'00' |  1   | Variable |    2     |
    ///
    ///   o  VER    protocol version: X'05'
    ///   o  REP    Reply field
    ///   o  RSV    RESERVED
    ///   o  ATYP   address type of following address
    ///   o  BND.ADDR       server bound address
    ///   o  BND.PORT       server bound port in network octet order
    fn send_reply(&mut self, rep: u8, atype: u8, addr: &[u8], port: u16) -> IoResult<()> {
        try!(self.stream.write(&[consts::VERSION_NUMBER, rep, 0x00, atype]));
        try!(self.stream.write(addr));
        try!(self.stream.write_be_u16(port));

        Ok(())
    }

    fn send_error_reply(&mut self, rep: u8) -> IoResult<()> {
        self.send_reply(rep, 0, &[0u8, 0, 0, 0], 0)
    }
}

pub fn handle_stream<'a>(stream: &'a mut TcpStream) -> () {
    let mut conn = SocksConnection { stream: stream };

    if conn.read_client_hello().is_err() {
        return;
    }

    conn.read_client_request();
}
