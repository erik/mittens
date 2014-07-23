# Architecture

There are two components to mittens, the `relay` and `server`. These are
usually (but not necessarily) on different boxes, with the `relay` being local
to the actual end user.

The `relay` speaks the SOCKSv5 protocol to the client, and then forwards
requests on to the `server` using encryption.

```
Client <-- plain text SOCKSv5 --> Relay <-- encrypted requests --> Server <-> Internet
```

**FIXME:** Currently this does not support perfect forward secrecy.

# Relay Protocol

The `relay` simply uses [SOCKSv5](http://www.rfcreader.com/#rfc1928) with no
authentication.

SOCKS doesn't actually provide useful encryption options, and `Client <->
Relay` should almost always be done on the same machine, so there shouldn't be
any issues not using authentication or encryption.

# Server Protocol

Each message exchanged between the `server` and the `relay` is prefixed with a
32 bit big endian unsigned integer followed by the content of the message.

The Relay is expected to obtain a copy of the server's cryptographic signature
public key and asymmetric encryption public key out of band.

- Sequence Legend
    * `<plain text>`
    * `[signed text]`
    * `{signed,encrypted text}`

### Sequence

### Handshake

- C->S: `C_HELLO: <random 128 byte nonce>`
- C<-S: `S_HELLO: <signed client nonce>`
- Client chooses a random 32 byte value to use as a symmetric encryption key
- C->S: `SET_KEY: <asymmetrically encrypted symmetric key>`
- Each subsequent interaction is encrypted with the given symmetric key.
- C<-S: `INFO: <version> <uptime> <num clients> <usage stats...>`

### Commands

After the handshake is successfully completed, the client may send any of the
following commands to the server.

- `PING: <timestamp>` -> `PONG: <timestamp>`
- `STOP`: Kill the connection
- `CONNECT: <host> <port>` -> `OK: <random symmetric encryption key>` | `FAIL: <error>`
