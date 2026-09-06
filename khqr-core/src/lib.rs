//! Core primitives for Cambodia's KHQR payment QR standard.
//!
//! KHQR payloads are EMVCo merchant-presented QR strings: flat ASCII made of
//! nested tag-length-value triples, closed by a CRC-16/CCITT-FALSE checksum.
//! This crate handles the encoding, decoding and validation of those payloads
//! and nothing else, so it stays usable from a CLI, a WASM bundle or a POS
//! terminal without pulling in an async runtime.
//!
//! # Examples
//!
//! ```
//! use khqr_core::parse_tlv;
//!
//! let fields = parse_tlv("0002010102115802KH")?;
//! assert_eq!(fields[1].tag, "01");
//! # Ok::<(), khqr_core::KhqrError>(())
//! ```
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
mod tlv;

pub use error::KhqrError;
pub use tlv::{format_tlv, parse_tlv, Tlv};
