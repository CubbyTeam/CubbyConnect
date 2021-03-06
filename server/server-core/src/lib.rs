//! Connects server and client with simple API.
//!
//! It uses TCP & UDP to connect and transfer data.
//! Also available for secure connection using TLS.
//!
//! # Features
//!
//! - fast & secure QUIC connection
//! - transfers data using protobuf
//! - pinging for heartbeat
//! - reconnection when internet is temporary disabled (in client)
//! - functional API that can be called in server & client
//! - connection to credential server for authentication
//! - version matching for compatability
//! - beautiful logging support

#[macro_use]
extern crate derive_builder;

pub use cubby_connect_server_macro::apply;

pub mod config;
pub mod fn_handler;
pub mod fn_layer;
pub mod handler;
pub mod layer;

mod protobuf {
    include!(concat!(env!("OUT_DIR"), "/sample.rs"));
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
