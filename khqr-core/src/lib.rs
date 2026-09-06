//! Core primitives for Cambodia's KHQR payment QR standard.
//!
//! KHQR payloads are EMVCo merchant-presented QR strings: flat ASCII made of
//! nested tag-length-value triples, closed by a CRC-16/CCITT-FALSE checksum.
//! This crate handles the encoding, decoding and validation of those payloads
//! and nothing else, so it stays usable from a CLI, a WASM bundle or a POS
//! terminal without pulling in an async runtime.
#![forbid(unsafe_code)]
#![warn(missing_docs)]
