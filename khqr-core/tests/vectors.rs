//! Structural checks on the reference vectors themselves, independent of the codec.

mod common;

#[test]
fn vectors_are_well_formed() {
    for qr in common::ALL {
        assert!(qr.is_ascii(), "vector is not ascii: {qr}");
        assert!(
            qr.starts_with("000201"),
            "missing payload format indicator: {qr}"
        );

        let (body, crc) = qr.split_at(qr.len() - 4);
        assert!(body.ends_with("6304"), "crc tag is not last: {qr}");
        assert_eq!(crc.len(), 4, "crc is not 4 characters: {crc}");
        assert!(
            crc.bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_lowercase()),
            "crc is not 4 uppercase hex chars: {crc}"
        );
    }
}
