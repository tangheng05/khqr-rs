//! Phase 2: the checksum must reproduce the published values.

mod common;

use khqr_core::{append_crc, crc16_ccitt_false, verify_crc};

/// The trailing checksum of each vector in [`common::ALL`], in order.
const EXPECTED: [&str; 4] = ["6894", "33E1", "9ACF", "9FBD"];

/// Splits off the four character checksum, leaving the body ending in `6304`.
fn body(qr: &str) -> &str {
    &qr[..qr.len() - 4]
}

fn with_last_char_changed(qr: &str) -> String {
    let replacement = if qr.ends_with('0') { '1' } else { '0' };
    format!("{}{replacement}", &qr[..qr.len() - 1])
}

#[test]
fn checksums_match_the_published_values() {
    for (qr, expected) in common::ALL.iter().zip(EXPECTED) {
        let checksum = crc16_ccitt_false(body(qr).as_bytes());
        assert_eq!(format!("{checksum:04X}"), expected);
    }
}

#[test]
fn every_vector_verifies() {
    for qr in common::ALL {
        assert!(verify_crc(qr), "vector failed its own checksum: {qr}");
    }
}

#[test]
fn append_crc_rebuilds_every_vector() {
    for qr in common::ALL {
        let without_checksum = &qr[..qr.len() - 8];
        assert_eq!(append_crc(without_checksum), qr);
    }
}

#[test]
fn an_edited_body_is_detected() {
    for qr in common::ALL {
        let tampered = qr.replacen("5802KH", "5802KM", 1);

        assert_ne!(tampered, qr, "vector should contain a country code");
        assert!(!verify_crc(&tampered), "edit to the body went unnoticed");
    }
}

#[test]
fn an_edited_checksum_is_detected() {
    for qr in common::ALL {
        assert!(!verify_crc(&with_last_char_changed(qr)));
    }
}
