//! Phase 4: every vector must decode to its published fields.

mod common;

use khqr_core::{append_crc, decode, verify_crc, Currency, KhqrError, MerchantType, Tlv};

#[test]
fn the_documented_vector_decodes_field_by_field() {
    let decoded = decode(common::INDIVIDUAL_WITH_LABELS).expect("vector must decode");

    assert_eq!(decoded.merchant_type, MerchantType::Individual);
    assert_eq!(decoded.bakong_account_id, "john_smith@devb");
    assert_eq!(decoded.bill_number.as_deref(), Some("#INV-2003"));
    assert_eq!(decoded.store_label.as_deref(), Some("Coffee Klaing"));
    assert_eq!(decoded.terminal_label.as_deref(), Some("#2"));
    assert_eq!(decoded.payload_format_indicator, "01");
    assert_eq!(decoded.point_of_initiation_method, "12");
    assert_eq!(decoded.merchant_category_code, "5999");
    assert_eq!(decoded.transaction_currency, "116");
    assert_eq!(decoded.transaction_amount.as_deref(), Some("5000.0"));
    assert_eq!(decoded.country_code, "KH");
    assert_eq!(decoded.merchant_name, "jonh smith");
    assert_eq!(decoded.merchant_city, "Phnom Penh");
    assert_eq!(decoded.crc, "9ACF");
    assert_eq!(decoded.currency(), Some(Currency::Khr));
    assert_eq!(decoded.created_at_ms, Some(1_613_027_972_757));
    assert_eq!(decoded.expires_at_ms, None);
}

#[test]
fn every_vector_decodes() {
    for qr in common::ALL {
        let decoded = decode(qr).expect("vector must decode");

        assert_eq!(decoded.country_code, "KH");
        assert_eq!(decoded.crc.chars().count(), 4);
        assert!(!decoded.merchant_name.is_empty());
    }
}

#[test]
fn the_individual_vector_has_no_optional_data() {
    let decoded = decode(common::INDIVIDUAL_KHR_500).expect("vector must decode");

    assert_eq!(decoded.merchant_type, MerchantType::Individual);
    assert_eq!(decoded.bakong_account_id, "jonhsmith@nbcq");
    assert_eq!(decoded.transaction_amount.as_deref(), Some("500"));
    assert_eq!(decoded.acquiring_bank, None);
    assert_eq!(decoded.merchant_id, None);
    assert_eq!(decoded.mobile_number, None);
    assert!(decoded.unknown.is_empty());
}

#[test]
fn the_merchant_vector_flattens_its_account_template() {
    let decoded = decode(common::MERCHANT_KHR).expect("vector must decode");

    assert_eq!(decoded.merchant_type, MerchantType::Merchant);
    assert_eq!(decoded.merchant_id.as_deref(), Some("123456"));
    assert_eq!(decoded.acquiring_bank.as_deref(), Some("Dev Bank"));
    assert_eq!(decoded.mobile_number.as_deref(), Some("85512345678"));
    assert_eq!(decoded.transaction_amount, None);
    assert_eq!(decoded.point_of_initiation_method, "11");
}

#[test]
fn the_production_vector_survives_its_vendor_extension() {
    let decoded = decode(common::ABA_MERCHANT).expect("vector must decode");

    assert_eq!(decoded.merchant_type, MerchantType::Merchant);
    assert_eq!(decoded.bakong_account_id, "abaakhppxxx@abaa");
    assert_eq!(decoded.merchant_id.as_deref(), Some("125021214532846"));
    assert_eq!(decoded.acquiring_bank.as_deref(), Some("ABA Bank"));
    assert_eq!(decoded.merchant_category_code, "5987");
    assert_eq!(decoded.merchant_name, "OLD ME 25 CHAR WINNER IP2");
    assert_eq!(decoded.bill_number.as_deref(), Some("MC-REF-KH-15000"));
    assert_eq!(decoded.created_at_ms, Some(1_759_805_345_337));
    assert_eq!(decoded.expires_at_ms, Some(1_759_805_525_337));

    let extra: Vec<&str> = decoded
        .additional_unknown
        .iter()
        .map(|field| field.tag.as_str())
        .collect();
    assert_eq!(extra, ["68"]);
}

#[test]
fn a_unionpay_merchant_account_is_decoded() {
    let qr = append_crc(concat!(
        "000201",
        "010211",
        "1510UPI-123456",
        "29130009shop@aclb",
        "52045999",
        "5303116",
        "5802KH",
        "5904Shop",
        "6010Phnom Penh",
    ));

    let decoded = decode(&qr).expect("payload must decode");

    assert_eq!(decoded.union_pay_merchant.as_deref(), Some("UPI-123456"));
    assert!(decoded.unknown.is_empty());
}

#[test]
fn an_unknown_top_level_tag_is_kept() {
    let qr = append_crc(concat!(
        "000201",
        "010211",
        "29130009shop@aclb",
        "52045999",
        "5303116",
        "5802KH",
        "5904Shop",
        "6010Phnom Penh",
        "8004TEST",
    ));

    let decoded = decode(&qr).expect("payload must decode");

    assert_eq!(
        decoded.unknown,
        vec![Tlv {
            tag: "80".to_string(),
            value: "TEST".to_string()
        }]
    );
}

#[test]
fn a_broken_checksum_is_rejected() {
    for qr in common::ALL {
        let tampered = qr.replacen("5802KH", "5802KM", 1);

        assert!(decode(&tampered).is_err(), "tampered payload was accepted");
    }
}

#[test]
fn a_payload_without_a_checksum_is_rejected() {
    assert!(decode("0002010102115802KH").is_err());
    assert!(decode("").is_err());
}

#[test]
fn a_payload_without_an_account_template_is_rejected() {
    let qr = append_crc(concat!(
        "000201",
        "010211",
        "52045999",
        "5303116",
        "5802KH",
        "5904Shop",
        "6010Phnom Penh",
    ));

    assert!(decode(&qr).is_err());
}

/// Currency 116 appears in its proper place, then 840 is appended and the
/// checksum recomputed. A last-wins parser charges dollars instead of riel.
const DUPLICATE_CURRENCY: &str =
    "00020101021229130009shop@aclb5204599953031165402105802KH5904Shop6010Phnom Penh5303840630447A5";

/// Tag 29 names the victim, tag 30 is appended naming the attacker. A parser
/// that lets the later template win sends the money to the wrong account.
const BOTH_ACCOUNT_TAGS: &str = "00020101021129150011victim@aclb30170013attacker@aclb5204599953031165802KH5904Shop6010Phnom Penh63048DE9";

#[test]
fn a_repeated_tag_is_refused_even_with_a_valid_checksum() {
    for tampered in [DUPLICATE_CURRENCY, BOTH_ACCOUNT_TAGS] {
        assert!(
            verify_crc(tampered),
            "this test is pointless unless the checksum is valid"
        );

        match decode(tampered) {
            Err(KhqrError::DuplicateTag { .. }) => {}
            other => panic!("a repeated tag must be refused, got {other:?}"),
        }
    }
}

#[test]
fn the_two_account_tags_are_mutually_exclusive() {
    assert!(matches!(
        decode(BOTH_ACCOUNT_TAGS),
        Err(KhqrError::DuplicateTag { .. })
    ));
}

#[test]
fn a_repeated_unknown_tag_is_still_tolerated() {
    let qr = append_crc(concat!(
        "000201",
        "010211",
        "29130009shop@aclb",
        "52045999",
        "5303116",
        "5802KH",
        "5904Shop",
        "6010Phnom Penh",
        "8004ONE1",
        "8004TWO2",
    ));

    let decoded = decode(&qr).expect("unknown tags are not the decoder's business");

    assert_eq!(decoded.unknown.len(), 2);
}

#[test]
fn a_timestamp_must_be_digits() {
    let qr = append_crc(concat!(
        "000201",
        "010211",
        "29130009shop@aclb",
        "52045999",
        "5303116",
        "5802KH",
        "5904Shop",
        "6010Phnom Penh",
        "991700131739495778722",
    ));
    assert!(decode(&qr).is_ok());

    let signed = append_crc(concat!(
        "000201",
        "010211",
        "29130009shop@aclb",
        "52045999",
        "5303116",
        "5802KH",
        "5904Shop",
        "6010Phnom Penh",
        "99170013+173949577872",
    ));
    assert!(decode(&signed).is_err());
}
