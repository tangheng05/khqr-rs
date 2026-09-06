use alloc::format;
use alloc::string::String;
/// Computes CRC-16/CCITT-FALSE: polynomial `0x1021`, init `0xFFFF`, no
/// reflection, no final XOR.
///
/// KHQR runs it over the UTF-8 bytes of the whole payload, including the
/// literal `6304` that introduces the checksum field.
pub fn crc16_ccitt_false(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;

    for &byte in data {
        crc ^= u16::from(byte) << 8;

        for _ in 0..8 {
            crc = if crc & 0x8000 == 0 {
                crc << 1
            } else {
                (crc << 1) ^ 0x1021
            };
        }
    }

    crc
}

/// Closes a payload by appending `6304` and the checksum.
///
/// Pass everything before it, with no `63` tag of its own.
///
/// ```
/// let qr = khqr_core::append_crc("0002010102115802KH");
/// assert_eq!(qr, "0002010102115802KH6304A3A4");
/// ```
pub fn append_crc(payload: &str) -> String {
    let mut qr = String::with_capacity(payload.len() + 8);
    qr.push_str(payload);
    qr.push_str("6304");

    let checksum = crc16_ccitt_false(qr.as_bytes());
    format!("{qr}{checksum:04X}")
}

/// Reports whether a payload ends in a `6304` field matching its contents.
///
/// Hex is accepted in either case. Anything too short, or not ending in a
/// `6304` field, is invalid.
pub fn verify_crc(qr: &str) -> bool {
    let total = qr.chars().count();
    if total < 8 {
        return false;
    }

    // Split by character: a corrupt payload can end mid character.
    let body: String = qr.chars().take(total - 4).collect();
    if !body.ends_with("6304") {
        return false;
    }

    let found: String = qr.chars().skip(total - 4).collect();
    let checksum = crc16_ccitt_false(body.as_bytes());

    found.eq_ignore_ascii_case(&format!("{checksum:04X}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_seed_is_returned_for_empty_input() {
        assert_eq!(crc16_ccitt_false(b""), 0xFFFF);
    }

    #[test]
    fn the_check_value_matches_the_specification() {
        assert_eq!(crc16_ccitt_false(b"123456789"), 0x29B1);
    }

    #[test]
    fn a_single_byte_difference_changes_the_checksum() {
        assert_ne!(crc16_ccitt_false(b"5802KH"), crc16_ccitt_false(b"5802KM"));
    }

    #[test]
    fn append_then_verify_round_trips() {
        let qr = append_crc("0002010102115802KH");

        assert!(qr.starts_with("0002010102115802KH6304"));
        assert!(verify_crc(&qr));
    }

    #[test]
    fn verification_accepts_lower_case_hex() {
        let qr = append_crc("0002010102115802KH");
        let split = qr.len() - 4;
        let lowered = format!("{}{}", &qr[..split], qr[split..].to_lowercase());

        assert_ne!(lowered, qr);
        assert!(verify_crc(&lowered));
    }

    #[test]
    fn a_payload_without_a_checksum_field_is_rejected() {
        assert!(!verify_crc(""));
        assert!(!verify_crc("6304"));
        assert!(!verify_crc("0002010102115802KH"));
    }

    #[test]
    fn a_multibyte_tail_is_rejected_instead_of_panicking() {
        assert!(!verify_crc("6304ភ្នំពេញ"));
    }
}
