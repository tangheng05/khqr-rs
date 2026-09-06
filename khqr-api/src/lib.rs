//! Client for the Bakong Open API, the service behind KHQR payments.
//!
//! Payload encoding lives in `khqr-core`; this crate only talks to the network.
//! It never sleeps or retries on a timer, so a screen left open for an hour
//! cannot quietly burn your rate limit.
#![forbid(unsafe_code)]

// Every Bakong URL is https, so a build with no TLS backend compiles and then
// fails every request at runtime. Refuse it at compile time instead.
#[cfg(not(any(feature = "rustls-tls", feature = "native-tls")))]
compile_error!(
    "khqr-api needs a TLS backend: enable either the `rustls-tls` or the `native-tls` feature"
);

mod backoff;
mod client;
mod error;
mod model;

pub use backoff::Backoff;
pub use client::BakongClient;
pub use error::ApiError;
pub use model::{Environment, SourceInfo, Transaction, TxStatus};
