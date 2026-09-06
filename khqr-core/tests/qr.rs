//! Phase 5: the payment handle and the rendered images.

mod common;

use khqr_core::md5;

#[test]
fn the_published_handles_are_reproduced() {
    assert_eq!(
        md5(common::INDIVIDUAL_KHR_500),
        common::INDIVIDUAL_KHR_500_MD5
    );
    assert_eq!(md5(common::MERCHANT_KHR), common::MERCHANT_KHR_MD5);
}

#[test]
fn a_handle_is_thirty_two_lower_case_hex_characters() {
    let handle = md5(common::ABA_MERCHANT);

    assert_eq!(handle.len(), 32);
    assert!(handle
        .chars()
        .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
}

#[cfg(feature = "image")]
mod images {
    use super::common;
    use khqr_core::{to_base64_uri, to_png, to_svg};

    /// Width and height from the PNG header, which starts at byte 16.
    fn dimensions(png: &[u8]) -> (u32, u32) {
        let width = u32::from_be_bytes(png[16..20].try_into().expect("png header is 4 bytes"));
        let height = u32::from_be_bytes(png[20..24].try_into().expect("png header is 4 bytes"));

        (width, height)
    }

    #[test]
    fn png_output_is_a_square_png_of_at_least_the_requested_size() {
        let png = to_png(common::INDIVIDUAL_KHR_500, 320).expect("vector must render");
        let (width, height) = dimensions(&png);

        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(width, height);
        assert!(width >= 320, "asked for 320, got {width}");
    }

    #[test]
    fn a_zero_size_is_rejected() {
        assert!(to_png(common::INDIVIDUAL_KHR_500, 0).is_err());
    }

    #[test]
    fn svg_output_is_an_svg() {
        let svg = to_svg(common::INDIVIDUAL_KHR_500).expect("vector must render");

        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn a_data_uri_carries_the_png() {
        let uri = to_base64_uri(common::INDIVIDUAL_KHR_500, 320).expect("vector must render");

        assert!(uri.starts_with("data:image/png;base64,"));
        assert!(uri.len() > 1000);
    }

    #[test]
    fn every_vector_renders() {
        for qr in common::ALL {
            assert!(to_png(qr, 256).is_ok());
            assert!(to_svg(qr).is_ok());
        }
    }
}
