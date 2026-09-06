//! Core primitives for Cambodia's KHQR payment QR standard.
//!
//! KHQR payloads are EMVCo merchant-presented QR strings: nested
//! tag-length-value triples closed by a CRC-16 checksum. No async runtime, no
//! HTTP, so this stays usable from a CLI, a WASM bundle or a POS terminal.
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod builder;
mod crc;
mod decoder;
mod error;
mod hash;
#[cfg(feature = "image")]
mod qr;
mod tlv;
mod types;

pub use builder::{Khqr, KhqrBuilder};
pub use crc::{append_crc, crc16_ccitt_false, verify_crc};
pub use decoder::{decode, DecodedKhqr};
pub use error::KhqrError;
pub use hash::md5;
#[cfg(feature = "image")]
pub use qr::{to_base64_uri, to_png, to_svg};
pub use tlv::{format_tlv, parse_tlv, Tlv};
pub use types::{Currency, MerchantType};
