use crate::{crc16_ccitt_false, parse_tlv, verify_crc, Currency, KhqrError, MerchantType, Tlv};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

const ADDITIONAL_TAGS: [&str; 6] = ["01", "02", "03", "05", "07", "08"];

/// Every field of a KHQR payload, with the nested templates flattened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedKhqr {
    pub payload_format_indicator: String,
    pub point_of_initiation_method: String,
    pub merchant_type: MerchantType,
    pub bakong_account_id: String,
    pub account_information: Option<String>,
    pub merchant_id: Option<String>,
    pub acquiring_bank: Option<String>,
    pub union_pay_merchant: Option<String>,
    pub merchant_category_code: String,
    pub transaction_currency: String,
    pub transaction_amount: Option<String>,
    pub country_code: String,
    pub merchant_name: String,
    pub merchant_city: String,
    pub bill_number: Option<String>,
    pub mobile_number: Option<String>,
    pub store_label: Option<String>,
    pub reference_label: Option<String>,
    pub terminal_label: Option<String>,
    pub purpose_of_transaction: Option<String>,
    pub language_preference: Option<String>,
    pub merchant_name_alternate: Option<String>,
    pub merchant_city_alternate: Option<String>,
    pub created_at_ms: Option<u64>,
    pub expires_at_ms: Option<u64>,
    pub crc: String,
    /// Top level tags this crate does not recognise.
    pub unknown: Vec<Tlv>,
    /// Tag `62` sub-tags this crate does not recognise.
    pub additional_unknown: Vec<Tlv>,
}

impl DecodedKhqr {
    /// The currency, if tag `53` held a code this crate knows.
    pub fn currency(&self) -> Option<Currency> {
        match self.transaction_currency.as_str() {
            "116" => Some(Currency::Khr),
            "840" => Some(Currency::Usd),
            _ => None,
        }
    }

    /// Whether the payload carries an amount, and so is single use.
    pub fn is_dynamic(&self) -> bool {
        self.transaction_amount.is_some()
    }
}

/// Reads a complete payload, checksum and all.
///
/// Unknown tags are kept rather than rejected, since production QRs carry
/// vendor extensions. A checksum that does not match is always an error.
pub fn decode(qr: &str) -> Result<DecodedKhqr, KhqrError> {
    let fields = parse_tlv(qr)?;

    let crc = match fields.last() {
        Some(field) if field.tag == "63" => field.value.clone(),
        _ => return Err(missing("checksum")),
    };

    if !verify_crc(qr) {
        let expected = checksum_of(qr, crc.chars().count());

        return Err(KhqrError::ChecksumMismatch {
            found: crc,
            expected,
        });
    }

    let mut payload_format_indicator = None;
    let mut point_of_initiation_method = None;
    let mut account = None;
    let mut union_pay_merchant = None;
    let mut merchant_category_code = None;
    let mut transaction_currency = None;
    let mut transaction_amount = None;
    let mut country_code = None;
    let mut merchant_name = None;
    let mut merchant_city = None;
    let mut additional = None;
    let mut language = None;
    let mut timestamps = None;
    let mut unknown = Vec::new();

    for field in fields {
        let tag = field.tag.clone();

        let taken = match tag.as_str() {
            "00" => payload_format_indicator.replace(field.value).is_some(),
            "01" => point_of_initiation_method.replace(field.value).is_some(),
            "15" => union_pay_merchant.replace(field.value).is_some(),
            // 29 and 30 share one slot, so a payload carrying both is refused
            // rather than quietly routing to whichever came last.
            "29" => account
                .replace((MerchantType::Individual, parse_tlv(&field.value)?))
                .is_some(),
            "30" => account
                .replace((MerchantType::Merchant, parse_tlv(&field.value)?))
                .is_some(),
            "52" => merchant_category_code.replace(field.value).is_some(),
            "53" => transaction_currency.replace(field.value).is_some(),
            "54" => transaction_amount.replace(field.value).is_some(),
            "58" => country_code.replace(field.value).is_some(),
            "59" => merchant_name.replace(field.value).is_some(),
            "60" => merchant_city.replace(field.value).is_some(),
            "62" => additional.replace(parse_tlv(&field.value)?).is_some(),
            "64" => language.replace(parse_tlv(&field.value)?).is_some(),
            "99" => timestamps.replace(parse_tlv(&field.value)?).is_some(),
            "63" => false,
            _ => {
                unknown.push(field);
                false
            }
        };

        if taken {
            return Err(KhqrError::DuplicateTag { tag });
        }
    }

    let additional = additional.unwrap_or_default();
    let language = language.unwrap_or_default();
    let timestamps = timestamps.unwrap_or_default();

    let (merchant_type, account_fields) = account.ok_or_else(|| missing("account template"))?;
    let account_detail = value_of(&account_fields, "01");
    let (account_information, merchant_id) = match merchant_type {
        MerchantType::Individual => (account_detail, None),
        MerchantType::Merchant => (None, account_detail),
    };

    let additional_unknown = additional
        .iter()
        .filter(|field| !ADDITIONAL_TAGS.contains(&field.tag.as_str()))
        .cloned()
        .collect();

    Ok(DecodedKhqr {
        payload_format_indicator: payload_format_indicator
            .ok_or_else(|| missing("payload format indicator"))?,
        point_of_initiation_method: point_of_initiation_method
            .ok_or_else(|| missing("point of initiation method"))?,
        merchant_type,
        bakong_account_id: value_of(&account_fields, "00")
            .ok_or_else(|| missing("bakong account id"))?,
        account_information,
        merchant_id,
        acquiring_bank: value_of(&account_fields, "02"),
        union_pay_merchant,
        merchant_category_code: merchant_category_code
            .ok_or_else(|| missing("merchant category code"))?,
        transaction_currency: transaction_currency
            .ok_or_else(|| missing("transaction currency"))?,
        transaction_amount,
        country_code: country_code.ok_or_else(|| missing("country code"))?,
        merchant_name: merchant_name.ok_or_else(|| missing("merchant name"))?,
        merchant_city: merchant_city.ok_or_else(|| missing("merchant city"))?,
        bill_number: value_of(&additional, "01"),
        mobile_number: value_of(&additional, "02"),
        store_label: value_of(&additional, "03"),
        reference_label: value_of(&additional, "05"),
        terminal_label: value_of(&additional, "07"),
        purpose_of_transaction: value_of(&additional, "08"),
        language_preference: value_of(&language, "00"),
        merchant_name_alternate: value_of(&language, "01"),
        merchant_city_alternate: value_of(&language, "02"),
        created_at_ms: millis(&timestamps, "00", "creation timestamp")?,
        expires_at_ms: millis(&timestamps, "01", "expiration timestamp")?,
        crc,
        unknown,
        additional_unknown,
    })
}

fn missing(field: &'static str) -> KhqrError {
    KhqrError::MissingField { field }
}

fn value_of(fields: &[Tlv], tag: &str) -> Option<String> {
    fields
        .iter()
        .find(|field| field.tag == tag)
        .map(|field| field.value.clone())
}

fn millis(fields: &[Tlv], tag: &str, field: &'static str) -> Result<Option<u64>, KhqrError> {
    match value_of(fields, tag) {
        Some(value) => {
            if !value.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(KhqrError::InvalidField { field, value });
            }

            value
                .parse()
                .map(Some)
                .map_err(|_| KhqrError::InvalidField { field, value })
        }
        None => Ok(None),
    }
}

fn checksum_of(qr: &str, checksum_length: usize) -> String {
    let kept = qr.chars().count().saturating_sub(checksum_length);
    let body: String = qr.chars().take(kept).collect();

    format!("{:04X}", crc16_ccitt_false(body.as_bytes()))
}
