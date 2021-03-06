#![deny(
    clippy::all,
    clippy::perf,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]
#![forbid(unsafe_code)]

macro_rules! next {
    ($iter:expr) => {
        $iter.next().ok_or(Error::new_parse("unexpected eof"))?
    };
}

macro_rules! next_parse {
    ($iter:expr) => {
        $iter
            .next()
            .ok_or(Error::new_parse("unexpected eof"))?
            .parse()
            .map_err(Error::new_parse)?
    };
}

mod client;
pub use self::client::Client;

mod connection;
pub(crate) use self::connection::{Connection, Request};

mod error;
pub use self::error::{Error, ErrorKind, Result};

pub mod models;

mod packet;
pub(crate) use self::packet::Packet;
