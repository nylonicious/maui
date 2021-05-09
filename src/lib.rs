#![deny(
    clippy::all,
    clippy::perf,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]
#![forbid(unsafe_code)]

#[macro_use]
macro_rules! next {
    ($iter:expr) => {
        $iter.next().ok_or(Error::new_parse("unexpected eof"))?
    };
}

mod client;
pub use self::client::Client;

mod connection;
pub(crate) use self::connection::{Connection, Request};

mod error;
pub use self::error::{Error, Result};

pub mod models;

mod packet;
pub(crate) use self::packet::Packet;
