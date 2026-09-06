use crate::KhqrError;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};
use qrcode::render::svg;
use qrcode::{Color, QrCode};

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const QUIET_ZONE: u32 = 4;
const MAX_SIZE: u32 = 4096;

/// Renders the payload as a PNG at least `size` pixels wide.
///
/// The side is rounded up to a whole number of pixels per module, so the image
/// stays sharp rather than being resampled.
pub fn to_png(qr: &str, size: u32) -> Result<Vec<u8>, KhqrError> {
    if size == 0 || size > MAX_SIZE {
        return Err(render(format!(
            "image size must be between 1 and {MAX_SIZE} pixels"
        )));
    }

    let code = encode(qr)?;
    let modules = code.width() as u32;
    let across = modules + QUIET_ZONE * 2;
    let scale = size.div_ceil(across).max(1);
    let side = across * scale;

    let mut pixels = vec![u8::MAX; (side * side) as usize];
    for (index, color) in code.to_colors().iter().enumerate() {
        if *color != Color::Dark {
            continue;
        }

        let left = (index as u32 % modules + QUIET_ZONE) * scale;
        let top = (index as u32 / modules + QUIET_ZONE) * scale;

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
    Ok(encode(qr)?.render::<svg::Color>().build())
}

/// Renders the payload as a PNG data URI, ready for an `img` tag.
pub fn to_base64_uri(qr: &str, size: u32) -> Result<String, KhqrError> {
    let png = to_png(qr, size)?;

    Ok(format!("data:image/png;base64,{}", base64(&png)))
}

fn encode(qr: &str) -> Result<QrCode, KhqrError> {
    QrCode::new(qr.as_bytes()).map_err(|error| render(error.to_string()))
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
