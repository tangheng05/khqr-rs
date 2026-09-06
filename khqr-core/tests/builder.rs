//! Phase 3: the builder must reproduce the published vectors exactly.
//!
//! Vectors 1 and 2 only. Vector 3 carries a fractional riel amount the builder
//! normalises, and vector 4 carries a vendor sub-tag the builder cannot emit.

mod common;

use khqr_core::{verify_crc, Currency, Khqr, KhqrBuilder, KhqrError, MerchantType};

const CREATED_AT: u64 = 1_739_495_778_722;

#[test]
fn the_individual_vector_is_rebuilt() {
    let qr = Khqr::individual("jonhsmith@nbcq")
        .merchant_name("Jonh Smith")
        .merchant_city("PHNOM PENH")
        .amount(500.0)
        .created_at_ms(CREATED_AT)
        .build()
        .expect("vector fields are valid")
        .to_qr_string()
        .expect("vector must serialise");

    assert_eq!(qr, common::INDIVIDUAL_KHR_500);
}

#[test]
fn the_merchant_vector_is_rebuilt() {
    let qr = Khqr::merchant("jonhsmith@nbcq")
        .merchant_id("123456")
        .acquiring_bank("Dev Bank")
        .merchant_name("Jonh Smith")
        .merchant_city("Siem Reap")
        .mobile_number("85512345678")
        .created_at_ms(CREATED_AT)
        .build()
        .expect("vector fields are valid")
        .to_qr_string()
        .expect("vector must serialise");

    assert_eq!(qr, common::MERCHANT_KHR);
}

#[test]
fn an_amount_makes_the_payload_dynamic() {
    let dynamic = Khqr::individual("shop@aclb")
        .merchant_name("Shop")
        .merchant_city("Phnom Penh")
        .amount(1.0)
        .build()
        .expect("fields are valid");

    let static_qr = Khqr::individual("shop@aclb")
        .merchant_name("Shop")
        .merchant_city("Phnom Penh")
        .build()
        .expect("fields are valid");

    assert!(dynamic.is_dynamic());
    assert!(!static_qr.is_dynamic());
    assert!(dynamic.to_qr_string().unwrap().starts_with("000201010212"));
    assert!(static_qr
        .to_qr_string()
        .unwrap()
        .starts_with("000201010211"));
}

#[test]
fn built_payloads_carry_a_valid_checksum() {
    let qr = Khqr::merchant("shop@aclb")
        .merchant_name("Shop")
        .merchant_city("Phnom Penh")
        .currency(Currency::Usd)
        .amount(0.1)
        .build()
        .expect("fields are valid")
        .to_qr_string()
        .expect("payload must serialise");

    assert!(verify_crc(&qr));
    assert!(qr.contains("5303840"));
    assert!(qr.contains("54040.10"));
}

#[test]
fn the_merchant_type_picks_the_account_tag() {
    let individual = Khqr::individual("shop@aclb")
        .merchant_name("Shop")
        .merchant_city("Phnom Penh")
        .build()
        .expect("fields are valid");

    assert_eq!(individual.merchant_type(), MerchantType::Individual);
    assert!(individual.to_qr_string().unwrap().contains("2913"));
}

#[test]
fn a_khmer_alternate_name_is_accepted() {
    let qr = Khqr::individual("shop@aclb")
        .merchant_name("Shop")
        .merchant_city("Phnom Penh")
        .alternate_language("KM", "ហាង", "ភ្នំពេញ")
        .build()
        .expect("fields are valid")
        .to_qr_string()
        .expect("payload must serialise");

    assert!(qr.contains("64240002KM0103ហាង0207ភ្នំពេញ"));
    assert!(verify_crc(&qr));
}

#[test]
fn an_account_id_must_name_a_bank() {
    for account_id in ["jonhsmith", "@nbcq", "jonhsmith@", "a@b@c"] {
        let result = Khqr::individual(account_id)
            .merchant_name("Shop")
            .merchant_city("Phnom Penh")
            .build();

        assert!(result.is_err(), "{account_id:?} should be rejected");
    }
}

#[test]
fn the_merchant_name_and_city_are_capped() {
    let long_name = Khqr::individual("shop@aclb")
        .merchant_name("A".repeat(26))
        .merchant_city("Phnom Penh")
        .build()
        .unwrap_err();

    assert_eq!(
        long_name,
        KhqrError::FieldTooLong {
            field: "merchant name",
            chars: 26,
            max: 25
        }
    );

    let long_city = Khqr::individual("shop@aclb")
        .merchant_name("Shop")
        .merchant_city("A".repeat(16))
        .build()
        .unwrap_err();

    assert_eq!(
        long_city,
        KhqrError::FieldTooLong {
            field: "merchant city",
            chars: 16,
            max: 15
        }
    );
}

#[test]
fn the_name_and_city_are_required() {
    assert_eq!(
        Khqr::individual("shop@aclb")
            .merchant_city("Phnom Penh")
            .build()
            .unwrap_err(),
        KhqrError::MissingField {
            field: "merchant name"
        }
    );

    assert_eq!(
        Khqr::individual("shop@aclb")
            .merchant_name("Shop")
            .build()
            .unwrap_err(),
        KhqrError::MissingField {
            field: "merchant city"
        }
    );
}

#[test]
fn a_negative_amount_is_rejected() {
    let result = Khqr::individual("shop@aclb")
        .merchant_name("Shop")
        .merchant_city("Phnom Penh")
        .amount(-1.0)
        .build();

    assert!(result.is_err());
}

#[test]
fn a_unionpay_merchant_account_is_written_to_tag_15() {
    let qr = Khqr::merchant("shop@aclb")
        .union_pay_merchant("UPI-123456")
        .merchant_name("Shop")
        .merchant_city("Phnom Penh")
        .build()
        .expect("fields are valid")
        .to_qr_string()
        .expect("payload must serialise");

    assert!(qr.contains("1510UPI-123456"));
    assert!(qr.starts_with("0002010102111510UPI-123456"));
    assert!(verify_crc(&qr));
}

#[test]
fn the_specification_field_limits_are_enforced() {
    let base = || {
        Khqr::individual("shop@aclb")
            .merchant_name("Shop")
            .merchant_city("Phnom Penh")
    };

    let cases: Vec<(&str, KhqrBuilder)> = vec![
        (
            "account id",
            Khqr::individual(format!("{}@aclb", "a".repeat(28)))
                .merchant_name("Shop")
                .merchant_city("Phnom Penh"),
        ),
        (
            "account information",
            base().account_information("a".repeat(33)),
        ),
        ("acquiring bank", base().acquiring_bank("a".repeat(33))),
        ("bill number", base().bill_number("a".repeat(26))),
        ("mobile number", base().mobile_number("a".repeat(26))),
        ("store label", base().store_label("a".repeat(26))),
        ("reference label", base().reference_label("a".repeat(26))),
        ("terminal label", base().terminal_label("a".repeat(26))),
        (
            "purpose of transaction",
            base().purpose_of_transaction("a".repeat(26)),
        ),
        (
            "unionpay merchant",
            base().union_pay_merchant("a".repeat(100)),
        ),
    ];

    for (field, builder) in cases {
        match builder.build() {
            Err(KhqrError::FieldTooLong {
                field: reported, ..
            }) => {
                assert_eq!(reported, field);
            }
            other => panic!("{field} should have been rejected, got {other:?}"),
        }
    }
}

#[test]
fn negative_zero_is_not_a_valid_amount() {
    let result = Khqr::individual("shop@aclb")
        .merchant_name("Shop")
        .merchant_city("Phnom Penh")
        .amount(-0.0)
        .build();

    assert!(result.is_err(), "-0.0 would write an amount of \"-0\"");
}

#[test]
fn ties_round_away_from_zero_like_the_reference_sdk() {
    let cases = [
        (Currency::Khr, 500.5, "501"),
        (Currency::Khr, 501.5, "502"),
        (Currency::Khr, 500.7, "501"),
        (Currency::Khr, 5000.0, "5000"),
        (Currency::Usd, 0.125, "0.13"),
        (Currency::Usd, 0.1, "0.10"),
        (Currency::Usd, 1.5, "1.50"),
        (Currency::Usd, 1234.5, "1234.50"),
    ];

    for (currency, amount, written) in cases {
        let qr = Khqr::individual("shop@aclb")
            .merchant_name("Shop")
            .merchant_city("Phnom Penh")
            .currency(currency)
            .amount(amount)
            .build()
            .expect("fields are valid")
            .to_qr_string()
            .expect("payload must serialise");

        let expected = format!("54{:02}{written}", written.len());
        assert!(qr.contains(&expected), "{amount} should write {written}");
    }
}

#[test]
fn empty_and_control_characters_are_refused() {
    let base = || {
        Khqr::individual("shop@aclb")
            .merchant_name("Shop")
            .merchant_city("Phnom Penh")
    };

    assert!(base().merchant_name("").build().is_err());
    assert!(base().merchant_city("").build().is_err());
    assert!(base().merchant_name("Sh\nop").build().is_err());
    assert!(base().merchant_city("Phnom\u{0}Penh").build().is_err());
    assert!(base().bill_number("INV\t1").build().is_err());
}

#[test]
fn an_account_id_may_not_carry_spaces_or_control_characters() {
    for account_id in ["shop@aclb\n", " shop@aclb", "shop@ac lb", "shop@aclb "] {
        let result = Khqr::individual(account_id)
            .merchant_name("Shop")
            .merchant_city("Phnom Penh")
            .build();

        assert!(result.is_err(), "{account_id:?} should be rejected");
    }
}

#[test]
fn a_language_preference_must_be_two_letters() {
    let result = Khqr::individual("shop@aclb")
        .merchant_name("Shop")
        .merchant_city("Phnom Penh")
        .alternate_language("K1", "Shop", "Phnom Penh")
        .build();

    assert!(result.is_err());
}

#[test]
fn rounding_matches_the_reference_sdk_on_near_ties() {
    // Each of these is held as slightly below the half, so toFixed rounds
    // down. Scaling by 100 first turns them into exact ties and rounds up.
    let cases = [
        (0.015, "0.01"),
        (0.045, "0.04"),
        (0.615, "0.61"),
        (1.115, "1.11"),
        (2.675, "2.67"),
        (8.885, "8.88"),
        // Exactly representable, so a genuine tie: away from zero.
        (0.125, "0.13"),
        (1.5, "1.50"),
    ];

    for (amount, written) in cases {
        let qr = Khqr::individual("shop@aclb")
            .merchant_name("Shop")
            .merchant_city("Phnom Penh")
            .currency(Currency::Usd)
            .amount(amount)
            .build()
            .expect("fields are valid")
            .to_qr_string()
            .expect("payload must serialise");

        let expected = format!("54{:02}{written}", written.len());
        assert!(qr.contains(&expected), "{amount} should write {written}");
    }
}

#[test]
fn an_amount_that_rounds_to_nothing_is_refused() {
    for amount in [0.0, 0.001, 0.004] {
        let result = Khqr::individual("shop@aclb")
            .merchant_name("Shop")
            .merchant_city("Phnom Penh")
            .currency(Currency::Usd)
            .amount(amount)
            .build();

        assert!(result.is_err(), "{amount} would write a QR nobody can pay");
    }
}

#[test]
fn control_characters_are_refused_in_the_routing_fields() {
    let base = || {
        Khqr::merchant("shop@aclb")
            .merchant_name("Shop")
            .merchant_city("Phnom Penh")
    };

    assert!(base().merchant_id("MID\n99").build().is_err());
    assert!(base().account_information("AC\nCT").build().is_err());
    assert!(base().acquiring_bank("AB\u{0}C").build().is_err());
    assert!(base().union_pay_merchant("UP\nI").build().is_err());
    assert!(base().merchant_id("").build().is_err());
}

#[test]
fn lengths_count_utf16_units_like_the_javascript_sdk() {
    // One emoji is two UTF-16 code units, which is what String.length reports
    // and therefore what every other KHQR SDK writes into the length field.
    let qr = Khqr::individual("shop@aclb")
        .merchant_name("\u{1F600} Shop")
        .merchant_city("Phnom Penh")
        .build()
        .expect("fields are valid")
        .to_qr_string()
        .expect("payload must serialise");

    assert!(qr.contains("5907\u{1F600} Shop"), "{qr}");
    assert!(verify_crc(&qr));
}
