#![feature(globs)]

// XXX: This feels hacky.
pub use mittens::client;
pub use mittens::server;

mod mittens {
    pub mod client;
    pub mod server;
}
