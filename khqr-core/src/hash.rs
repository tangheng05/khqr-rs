use md5::{Digest, Md5};
use std::fmt::Write;

/// The MD5 handle used to poll a payment, 32 lower case hex characters.
///
/// Computed locally over the whole payload. It only identifies a transaction
/// for dynamic QRs; a static one has no amount and cannot be tracked this way.
pub fn md5(qr: &str) -> String {
    let digest = Md5::digest(qr.as_bytes());

    digest
        .iter()
        .fold(String::with_capacity(32), |mut out, byte| {
            let _ = write!(out, "{byte:02x}");
            out
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_empty_string_hashes_to_the_known_value() {
        assert_eq!(md5(""), "d41d8cd98f00b204e9800998ecf8427e");
    }

    #[test]
    fn leading_zeros_in_a_byte_are_kept() {
        assert_eq!(md5("jk").len(), 32);
        assert!(md5("a").starts_with("0cc175b9"));
    }
}
