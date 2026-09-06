//! Client for the Bakong Open API, the service behind KHQR payments.
//!
//! Payload encoding lives in `khqr-core`; this crate only talks to the network.
//! It never sleeps or retries on a timer, so a screen left open for an hour
//! cannot quietly burn your rate limit.
#![forbid(unsafe_code)]

mod backoff;
mod client;
mod error;
mod model;

pub use backoff::Backoff;
pub use client::BakongClient;
pub use error::ApiError;
pub use model::{Environment, SourceInfo, Transaction, TxStatus};
