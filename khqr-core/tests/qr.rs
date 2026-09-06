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
    use khqr_core::{
        to_base64_uri, to_png, to_png_with, to_svg, to_svg_with, ErrorCorrection, ImageOptions,
    };

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
    fn an_absurd_size_is_refused_rather_than_overflowing() {
        for size in [65_535, 100_000, u32::MAX] {
            assert!(
                to_png(common::INDIVIDUAL_KHR_500, size).is_err(),
                "size {size} must be refused"
            );
        }
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
    #[test]
    fn the_default_options_match_the_simple_call() {
        let plain = to_png(common::INDIVIDUAL_KHR_500, 320).expect("renders");
        let same = to_png_with(
            common::INDIVIDUAL_KHR_500,
            &ImageOptions {
                size: 320,
                ..ImageOptions::default()
            },
        )
        .expect("renders");

        assert_eq!(plain, same);
    }

    #[test]
    fn higher_error_correction_makes_a_denser_code() {
        let sizes: Vec<u32> = [
            ErrorCorrection::Low,
            ErrorCorrection::Medium,
            ErrorCorrection::Quartile,
            ErrorCorrection::High,
        ]
        .into_iter()
        .map(|error_correction| {
            let png = to_png_with(
                common::INDIVIDUAL_KHR_500,
                &ImageOptions {
                    size: 1,
                    quiet_zone: 0,
                    error_correction,
                },
            )
            .expect("renders");

            dimensions(&png).0
        })
        .collect();

        // One pixel per module, so the width is the module count.
        assert!(
            sizes.windows(2).all(|pair| pair[0] <= pair[1]),
            "expected non decreasing module counts, got {sizes:?}"
        );
        assert!(
            sizes[3] > sizes[0],
            "High must carry more modules than Low, got {sizes:?}"
        );
    }

    #[test]
    fn the_quiet_zone_can_be_removed() {
        let options = |quiet_zone| ImageOptions {
            size: 1,
            quiet_zone,
            ..ImageOptions::default()
        };

        let bare = to_png_with(common::INDIVIDUAL_KHR_500, &options(0)).expect("renders");
        let padded = to_png_with(common::INDIVIDUAL_KHR_500, &options(4)).expect("renders");

        assert_eq!(dimensions(&padded).0, dimensions(&bare).0 + 8);
    }

    #[test]
    fn an_overlay_preset_uses_high_correction() {
        assert_eq!(
            ImageOptions::with_overlay().error_correction,
            ErrorCorrection::High
        );

        let svg = to_svg_with(common::INDIVIDUAL_KHR_500, &ImageOptions::with_overlay())
            .expect("renders");
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn an_enormous_quiet_zone_is_refused() {
        let result = to_png_with(
            common::INDIVIDUAL_KHR_500,
            &ImageOptions {
                size: 4096,
                quiet_zone: 4096,
                ..ImageOptions::default()
            },
        );

        assert!(result.is_err());
    }
}
