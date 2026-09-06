//! Phase 7: the shapes crossing the FFI boundary.

use khqr_ffi::{
    decode, generate, md5, to_png, verify, Currency, KhqrError, KhqrOptions, MerchantType,
};

const VECTOR: &str = "00020101021229180014jonhsmith@nbcq52045999530311654035005802KH5910Jonh Smith6010PHNOM PENH99170013173949577872263046894";

fn options(merchant_type: MerchantType, account_id: &str) -> KhqrOptions {
    KhqrOptions {
        merchant_type,
        account_id: account_id.to_string(),
        merchant_name: "Jonh Smith".to_string(),
        merchant_city: "PHNOM PENH".to_string(),
        currency: None,
        amount: None,
        account_information: None,
        merchant_id: None,
        acquiring_bank: None,
        union_pay_merchant: None,
        merchant_category_code: None,
        country_code: None,
        bill_number: None,
        mobile_number: None,
        store_label: None,
        reference_label: None,
        terminal_label: None,
        purpose_of_transaction: None,
        language_preference: None,
        merchant_name_alternate: None,
        merchant_city_alternate: None,
        created_at_ms: None,
        expires_at_ms: None,
    }
}

#[test]
fn the_published_vector_is_rebuilt_across_the_boundary() {
    let built = generate(KhqrOptions {
        amount: Some(500.0),
        created_at_ms: Some(1_739_495_778_722),
        ..options(MerchantType::Individual, "jonhsmith@nbcq")
    })
    .expect("vector fields are valid");

    assert_eq!(built, VECTOR);
    assert_eq!(md5(built), "b1c250304b8594e4c6b53dd44791b57a");
}

#[test]
fn a_dollar_amount_keeps_its_decimals() {
    let built = generate(KhqrOptions {
        currency: Some(Currency::Usd),
        amount: Some(0.1),
        ..options(MerchantType::Merchant, "shop@aclb")
    })
    .expect("fields are valid");

    assert!(built.contains("54040.10"));
    assert!(built.contains("5303840"));
}

#[test]
fn decoding_flattens_every_template() {
    let decoded = decode(VECTOR.to_string()).expect("vector must decode");

    assert_eq!(decoded.merchant_type, MerchantType::Individual);
    assert_eq!(decoded.bakong_account_id, "jonhsmith@nbcq");
    assert_eq!(decoded.merchant_name, "Jonh Smith");
    assert_eq!(decoded.transaction_amount.as_deref(), Some("500"));
    assert_eq!(decoded.created_at_ms, Some(1_739_495_778_722));
    assert!(decoded.is_dynamic);
    assert!(decoded.unknown.is_empty());
}

#[test]
fn a_broken_checksum_is_rejected() {
    let tampered = VECTOR.replacen("5802KH", "5802KM", 1);

    assert!(!verify(tampered.clone()));
    assert!(decode(tampered).is_err());
}

#[test]
fn errors_cross_the_boundary_with_their_message() {
    let error = generate(options(MerchantType::Individual, "no-bank-here"))
        .expect_err("an account id without a bank is invalid");

    let KhqrError::Invalid { message } = error;
    assert!(
        message.contains("account id"),
        "unhelpful message: {message}"
    );
}

#[test]
fn images_render_through_the_boundary() {
    let png = to_png(VECTOR.to_string(), 320).expect("vector must render");

    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
}
