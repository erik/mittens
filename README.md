# mittens [![Build Status](https://travis-ci.org/erik/mittens.svg?branch=master)](https://travis-ci.org/erik/mittens)

Mittens is an implementation of the [SOCKS proxy](http://en.wikipedia.org/wiki/SOCKS) protocol written in Rust.

This application is more or less a proof of concept test for the [knuckle](https://github.com/erik/knuckle) cryptography library, and isn't quite ready for use yet. The end goal is a simple minimal setup SOCKS proxy that can be deployed to any server.

## Building

You'll need to grab [cargo](crates.io) to build and test mittens. After that, building the application is as simple as:

```bash
$ git clone git@github.com:erik/mittens.git
$ cd mittens
$ cargo build
```

## License
Mittens is distributed under the MIT license. See the LICENSE file in this directory for more information.
