//! Core primitives for Cambodia's KHQR payment QR standard.
//!
//! KHQR payloads are EMVCo merchant-presented QR strings: nested
//! tag-length-value triples closed by a CRC-16 checksum. No async runtime, no
//! HTTP, so this stays usable from a CLI, a WASM bundle or a POS terminal.
#![forbid(unsafe_code)]

mod builder;
mod crc;
mod decoder;
mod error;
mod tlv;
mod types;

pub use builder::{Khqr, KhqrBuilder};
pub use crc::{append_crc, crc16_ccitt_false, verify_crc};
pub use decoder::{decode, DecodedKhqr};
pub use error::KhqrError;
pub use tlv::{format_tlv, parse_tlv, Tlv};
pub use types::{Currency, MerchantType};
