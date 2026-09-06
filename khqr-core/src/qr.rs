use crate::KhqrError;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};
use qrcode::render::svg;
use qrcode::{Color, EcLevel, QrCode};

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const QUIET_ZONE: u32 = 4;
const MAX_SIZE: u32 = 4096;

/// How much of a covered or damaged code can still be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ErrorCorrection {
    /// About 7%.
    Low,
    /// About 15%.
    #[default]
    Medium,
    /// About 25%.
    Quartile,
    /// About 30%. Use this if you draw anything over the code.
    High,
}

impl From<ErrorCorrection> for EcLevel {
    fn from(level: ErrorCorrection) -> Self {
        match level {
            ErrorCorrection::Low => Self::L,
            ErrorCorrection::Medium => Self::M,
            ErrorCorrection::Quartile => Self::Q,
            ErrorCorrection::High => Self::H,
        }
    }
}

/// How to draw the code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageOptions {
    /// Smallest width in pixels. The SVG renderer ignores it.
    pub size: u32,
    pub error_correction: ErrorCorrection,
    /// Blank modules around the code. Zero if your layout already pads it.
    pub quiet_zone: u32,
}

impl Default for ImageOptions {
    fn default() -> Self {
        Self {
            size: 512,
            error_correction: ErrorCorrection::default(),
            quiet_zone: QUIET_ZONE,
        }
    }
}

impl ImageOptions {
    /// Defaults, but with the error correction a logo overlay needs.
    pub fn with_overlay() -> Self {
        Self {
            error_correction: ErrorCorrection::High,
            ..Self::default()
        }
    }
}

/// Renders the payload as a PNG at least `size` pixels wide.
///
/// The side is rounded up to a whole number of pixels per module, so the image
/// stays sharp rather than being resampled.
pub fn to_png(qr: &str, size: u32) -> Result<Vec<u8>, KhqrError> {
    to_png_with(
        qr,
        &ImageOptions {
            size,
            ..ImageOptions::default()
        },
    )
}

/// Renders a PNG with the error correction and quiet zone you choose.
pub fn to_png_with(qr: &str, options: &ImageOptions) -> Result<Vec<u8>, KhqrError> {
    let size = options.size;
    if size == 0 || size > MAX_SIZE {
        return Err(render(format!(
            "image size must be between 1 and {MAX_SIZE} pixels"
        )));
    }

    let quiet = options.quiet_zone.min(MAX_SIZE);
    let code = encode_with(qr, options.error_correction)?;
    let modules = code.width() as u32;
    let across = modules + quiet * 2;
    let scale = size.div_ceil(across).max(1);
    let side = across * scale;

    if side > MAX_SIZE {
        return Err(render(format!(
            "quiet zone of {quiet} makes the image larger than {MAX_SIZE} pixels"
        )));
    }

    let mut pixels = vec![u8::MAX; (side * side) as usize];
    for (index, color) in code.to_colors().iter().enumerate() {
        if *color != Color::Dark {
            continue;
        }

        let left = (index as u32 % modules + quiet) * scale;
        let top = (index as u32 / modules + quiet) * scale;

        for y in top..top + scale {
            let row = (y * side) as usize;
            pixels[row + left as usize..row + (left + scale) as usize].fill(0);
        }
    }

    let mut png = Vec::new();
    PngEncoder::new(&mut png)
        .write_image(&pixels, side, side, ExtendedColorType::L8)
        .map_err(|error| render(error.to_string()))?;

    Ok(png)
}

/// Renders the payload as an SVG, which scales to any size on its own.
pub fn to_svg(qr: &str) -> Result<String, KhqrError> {
    to_svg_with(qr, &ImageOptions::default())
}

/// Renders an SVG with those same options.
pub fn to_svg_with(qr: &str, options: &ImageOptions) -> Result<String, KhqrError> {
    Ok(encode_with(qr, options.error_correction)?
        .render::<svg::Color>()
        .quiet_zone(options.quiet_zone > 0)
        .build())
}

/// Renders the payload as a PNG data URI, ready for an `img` tag.
pub fn to_base64_uri(qr: &str, size: u32) -> Result<String, KhqrError> {
    to_base64_uri_with(
        qr,
        &ImageOptions {
            size,
            ..ImageOptions::default()
        },
    )
}

/// A PNG data URI with those same options.
pub fn to_base64_uri_with(qr: &str, options: &ImageOptions) -> Result<String, KhqrError> {
    let png = to_png_with(qr, options)?;

    Ok(format!("data:image/png;base64,{}", base64(&png)))
}

fn encode_with(qr: &str, level: ErrorCorrection) -> Result<QrCode, KhqrError> {
    QrCode::with_error_correction_level(qr.as_bytes(), level.into())
        .map_err(|error| render(error.to_string()))
}

fn render(reason: impl Into<String>) -> KhqrError {
    KhqrError::Render {
        reason: reason.into(),
    }
}

fn base64(data: &[u8]) -> String {
    let mut encoded = String::with_capacity(data.len().div_ceil(3) * 4);

    for chunk in data.chunks(3) {
        let bytes = [
            u32::from(chunk[0]),
            u32::from(chunk.get(1).copied().unwrap_or(0)),
            u32::from(chunk.get(2).copied().unwrap_or(0)),
        ];
        let packed = (bytes[0] << 16) | (bytes[1] << 8) | bytes[2];

        encoded.push(BASE64[((packed >> 18) & 63) as usize] as char);
        encoded.push(BASE64[((packed >> 12) & 63) as usize] as char);
        encoded.push(match chunk.len() {
            1 => '=',
            _ => BASE64[((packed >> 6) & 63) as usize] as char,
        });
        encoded.push(match chunk.len() {
            3 => BASE64[(packed & 63) as usize] as char,
            _ => '=',
        });
    }

    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_the_published_vectors() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foob"), "Zm9vYg==");
        assert_eq!(base64(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn base64_handles_bytes_outside_ascii() {
        assert_eq!(base64(&[0x00, 0xFF, 0x80]), "AP+A");
    }
}
