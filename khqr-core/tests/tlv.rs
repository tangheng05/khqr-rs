//! Phase 1: the TLV codec must survive the published vectors.

mod common;

use khqr_core::{format_tlv, parse_tlv, Tlv};

/// Tags whose value is itself a run of triples.
const TEMPLATE_TAGS: [&str; 5] = ["29", "30", "62", "64", "99"];

fn encode(fields: &[Tlv]) -> String {
    fields
        .iter()
        .map(|field| format_tlv(&field.tag, &field.value).expect("field must re-encode"))
        .collect()
}

#[test]
fn every_vector_round_trips() {
    for qr in common::ALL {
        let fields = parse_tlv(qr).expect("vector must parse");
        assert_eq!(encode(&fields), qr, "round trip changed the payload");
    }
}

#[test]
fn nested_templates_round_trip() {
    for qr in common::ALL {
        for field in parse_tlv(qr).expect("vector must parse") {
            if !TEMPLATE_TAGS.contains(&field.tag.as_str()) {
                continue;
            }

            let inner = parse_tlv(&field.value).expect("template must parse");
            assert!(
                !inner.is_empty(),
                "template {} decoded to nothing",
                field.tag
            );
            assert_eq!(
                encode(&inner),
                field.value,
                "template {} did not round trip",
                field.tag
            );
        }
    }
}

#[test]
fn crc_is_the_final_field() {
    for qr in common::ALL {
        let fields = parse_tlv(qr).expect("vector must parse");
        let last = fields.last().expect("vector must have fields");

        assert_eq!(last.tag, "63");
        assert_eq!(last.value.chars().count(), 4);
    }
}

#[test]
fn vendor_extensions_inside_a_template_are_just_more_fields() {
    let fields = parse_tlv(common::ABA_MERCHANT).expect("vector must parse");
    let additional = fields
        .iter()
        .find(|field| field.tag == "62")
        .expect("aba vector has a tag 62");

    let inner = parse_tlv(&additional.value).expect("template must parse");
    let tags: Vec<&str> = inner.iter().map(|field| field.tag.as_str()).collect();

    assert_eq!(tags, ["01", "68"]);
}
