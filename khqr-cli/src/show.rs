use khqr_core::{decode as decode_khqr, md5, verify_crc, DecodedKhqr};
use std::error::Error;

pub fn decode_payload(qr: &str) -> Result<DecodedKhqr, Box<dyn Error>> {
    Ok(decode_khqr(qr)?)
}

pub fn decode(qr: &str) -> Result<(), Box<dyn Error>> {
    let decoded = decode_payload(qr)?;

    let mut rows: Vec<(&str, String)> = vec![
        ("merchant type", format!("{:?}", decoded.merchant_type)),
        ("bakong account id", decoded.bakong_account_id.clone()),
        ("merchant name", decoded.merchant_name.clone()),
        ("merchant city", decoded.merchant_city.clone()),
        (
            "merchant category code",
            decoded.merchant_category_code.clone(),
        ),
        ("transaction currency", decoded.transaction_currency.clone()),
        ("country code", decoded.country_code.clone()),
        (
            "payload format indicator",
            decoded.payload_format_indicator.clone(),
        ),
        (
            "point of initiation",
            decoded.point_of_initiation_method.clone(),
        ),
        ("crc", decoded.crc.clone()),
        ("md5", md5(qr)),
    ];

    for (label, value) in [
        ("union pay merchant", &decoded.union_pay_merchant),
        ("account information", &decoded.account_information),
        ("merchant id", &decoded.merchant_id),
        ("acquiring bank", &decoded.acquiring_bank),
        ("transaction amount", &decoded.transaction_amount),
        ("bill number", &decoded.bill_number),
        ("mobile number", &decoded.mobile_number),
        ("store label", &decoded.store_label),
        ("reference label", &decoded.reference_label),
        ("terminal label", &decoded.terminal_label),
        ("purpose of transaction", &decoded.purpose_of_transaction),
        ("language preference", &decoded.language_preference),
        ("merchant name alternate", &decoded.merchant_name_alternate),
        ("merchant city alternate", &decoded.merchant_city_alternate),
    ] {
        if let Some(value) = value {
            rows.push((label, value.clone()));
        }
    }

    for (label, millis) in [
        ("created at", decoded.created_at_ms),
        ("expires at", decoded.expires_at_ms),
    ] {
        if let Some(millis) = millis {
            rows.push((label, millis.to_string()));
        }
    }

    for field in decoded.unknown.iter().chain(&decoded.additional_unknown) {
        rows.push(("unknown tag", format!("{} {}", field.tag, field.value)));
    }

    let width = rows.iter().map(|(label, _)| label.len()).max().unwrap_or(0);
    for (label, value) in rows {
        println!("{label:width$}  {value}");
    }

    Ok(())
}

pub fn verify(qr: &str) -> Result<(), Box<dyn Error>> {
    if !verify_crc(qr) {
        return Err("checksum does not match".into());
    }

    decode_payload(qr)?;
    println!("ok");

    Ok(())
}
