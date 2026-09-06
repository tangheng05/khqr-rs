//! The snippets printed in docs/, compiled so they cannot rot.

use khqr_core::{decode, format_tlv, md5, parse_tlv, verify_crc, Currency, Khqr, KhqrError};

const VECTOR: &str = "00020101021229180014jonhsmith@nbcq52045999530311654035005802KH5910Jonh Smith6010PHNOM PENH99170013173949577872263046894";

#[test]
fn getting_started_snippet() -> Result<(), KhqrError> {
    let qr = Khqr::individual("shop@aclb")
        .merchant_name("Coffee Klaing")
        .merchant_city("Phnom Penh")
        .amount(5000.0)
        .build()?
        .to_qr_string()?;

    let handle = md5(&qr);

    assert!(verify_crc(&qr));
    assert_eq!(handle.len(), 32);
    Ok(())
}

#[test]
fn generating_snippet() -> Result<(), KhqrError> {
    let now_ms = 1_739_495_778_722;
    let qr = Khqr::merchant("shop@aclb")
        .merchant_name("Coffee Klaing")
        .merchant_city("Phnom Penh")
        .merchant_id("125021214532846")
        .acquiring_bank("ACLEDA Bank")
        .currency(Currency::Usd)
        .amount(1.5)
        .bill_number("INV-2003")
        .terminal_label("POS-1")
        .created_at_ms(now_ms)
        .expires_at_ms(now_ms + 300_000)
        .build()?
        .to_qr_string()?;

    assert!(qr.contains("54041.50"));
    assert!(qr.contains("5303840"));
    Ok(())
}

#[test]
fn amount_formatting_table() -> Result<(), KhqrError> {
    let cases = [
        (Currency::Khr, 5000.0, "5000"),
        (Currency::Khr, 500.7, "501"),
        (Currency::Usd, 1.5, "1.50"),
        (Currency::Usd, 0.1, "0.10"),
    ];

    for (currency, amount, written) in cases {
        let qr = Khqr::individual("shop@aclb")
            .merchant_name("Shop")
            .merchant_city("Phnom Penh")
            .currency(currency)
            .amount(amount)
            .build()?
            .to_qr_string()?;

        assert!(
            qr.contains(&format_tlv("54", written)?),
            "{amount} became wrong"
        );
    }

    Ok(())
}

#[test]
fn khmer_alternate_language_snippet() -> Result<(), KhqrError> {
    let qr = Khqr::individual("shop@aclb")
        .merchant_name("Shop")
        .merchant_city("Phnom Penh")
        .alternate_language("KM", "កាហ្វេ ក្លាំង", "ភ្នំពេញ")
        .build()?
        .to_qr_string()?;

    assert!(verify_crc(&qr));
    Ok(())
}

// Uses Box<dyn Error>, which KhqrError only implements with std.
#[cfg(feature = "std")]
#[test]
fn decoding_snippets() -> Result<(), Box<dyn std::error::Error>> {
    let decoded = decode(VECTOR)?;

    assert_eq!(decoded.merchant_name, "Jonh Smith");
    assert_eq!(decoded.currency(), Some(Currency::Khr));

    let amount: f64 = decoded
        .transaction_amount
        .as_deref()
        .unwrap_or("0")
        .parse()?;
    assert_eq!(amount, 500.0);

    for field in decoded.unknown.iter().chain(&decoded.additional_unknown) {
        assert!(!field.tag.is_empty());
    }

    Ok(())
}

#[test]
fn tlv_round_trip_snippet() -> Result<(), KhqrError> {
    let fields = parse_tlv(VECTOR)?;
    let rebuilt: String = fields
        .iter()
        .map(|f| format_tlv(&f.tag, &f.value).expect("field re-encodes"))
        .collect();

    assert_eq!(rebuilt, VECTOR);
    Ok(())
}

#[cfg(feature = "image")]
#[test]
fn image_snippets() -> Result<(), KhqrError> {
    use khqr_core::{to_base64_uri, to_png, to_svg};

    let png = to_png(VECTOR, 512)?;
    let svg = to_svg(VECTOR)?;
    let uri = to_base64_uri(VECTOR, 512)?;

    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    assert!(svg.contains("<svg"));
    assert!(uri.starts_with("data:image/png;base64,"));
    Ok(())
}
