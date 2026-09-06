use crate::{append_crc, format_tlv, Currency, KhqrError, MerchantType};
use alloc::format;
use alloc::string::{String, ToString};

const MAX_NAME: usize = 25;
const MAX_CITY: usize = 15;
const MAX_AMOUNT: usize = 13;
const MAX_ACCOUNT: usize = 32;
const MAX_LABEL: usize = 25;
const MAX_TIMESTAMP: usize = 13;
const MAX_UNION_PAY: usize = 99;
const DEFAULT_CATEGORY_CODE: &str = "5999";
const DEFAULT_COUNTRY_CODE: &str = "KH";

#[derive(Debug, Clone, Default)]
struct AdditionalData {
    bill_number: Option<String>,
    mobile_number: Option<String>,
    store_label: Option<String>,
    reference_label: Option<String>,
    terminal_label: Option<String>,
    purpose: Option<String>,
}

impl AdditionalData {
    fn fields(&self) -> [(&'static str, &'static str, &Option<String>); 6] {
        [
            ("01", "bill number", &self.bill_number),
            ("02", "mobile number", &self.mobile_number),
            ("03", "store label", &self.store_label),
            ("05", "reference label", &self.reference_label),
            ("07", "terminal label", &self.terminal_label),
            ("08", "purpose of transaction", &self.purpose),
        ]
    }
}

#[derive(Debug, Clone)]
struct AlternateLanguage {
    preference: String,
    name: String,
    city: String,
}

/// A validated KHQR payload, ready to serialise.
#[derive(Debug, Clone)]
pub struct Khqr {
    merchant_type: MerchantType,
    account_id: String,
    account_detail: Option<String>,
    acquiring_bank: Option<String>,
    union_pay_merchant: Option<String>,
    merchant_category_code: String,
    currency: Currency,
    amount: Option<String>,
    country_code: String,
    merchant_name: String,
    merchant_city: String,
    additional: AdditionalData,
    alternate_language: Option<AlternateLanguage>,
    created_at_ms: Option<u64>,
    expires_at_ms: Option<u64>,
}

impl Khqr {
    /// Starts a payload for a personal account, written to tag `29`.
    pub fn individual(account_id: impl Into<String>) -> KhqrBuilder {
        KhqrBuilder::new(MerchantType::Individual, account_id.into())
    }

    /// Starts a payload for a business account, written to tag `30`.
    pub fn merchant(account_id: impl Into<String>) -> KhqrBuilder {
        KhqrBuilder::new(MerchantType::Merchant, account_id.into())
    }

    /// Whether the account lives in tag `29` or tag `30`.
    pub fn merchant_type(&self) -> MerchantType {
        self.merchant_type
    }

    /// Whether the payload carries an amount, and so is single use.
    pub fn is_dynamic(&self) -> bool {
        self.amount.is_some()
    }

    /// Writes the payload out, checksum included.
    ///
    /// Fails if a template grew past the 99 characters a length field can
    /// express, which optional labels can do even when each one is valid.
    pub fn to_qr_string(&self) -> Result<String, KhqrError> {
        let point_of_initiation = if self.is_dynamic() { "12" } else { "11" };

        let mut payload = format_tlv("00", "01")?;
        payload.push_str(&format_tlv("01", point_of_initiation)?);

        if let Some(union_pay) = &self.union_pay_merchant {
            payload.push_str(&format_tlv("15", union_pay)?);
        }

        payload.push_str(&format_tlv(
            self.merchant_type.tag(),
            &self.account_template()?,
        )?);
        payload.push_str(&format_tlv("52", &self.merchant_category_code)?);
        payload.push_str(&format_tlv("53", self.currency.code())?);

        if let Some(amount) = &self.amount {
            payload.push_str(&format_tlv("54", amount)?);
        }

        payload.push_str(&format_tlv("58", &self.country_code)?);
        payload.push_str(&format_tlv("59", &self.merchant_name)?);
        payload.push_str(&format_tlv("60", &self.merchant_city)?);

        let additional = self.additional_template()?;
        if !additional.is_empty() {
            payload.push_str(&format_tlv("62", &additional)?);
        }

        if let Some(language) = &self.alternate_language {
            payload.push_str(&format_tlv("64", &language_template(language)?)?);
        }

        let timestamp = self.timestamp_template()?;
        if !timestamp.is_empty() {
            payload.push_str(&format_tlv("99", &timestamp)?);
        }

        Ok(append_crc(&payload))
    }

    fn account_template(&self) -> Result<String, KhqrError> {
        let mut template = format_tlv("00", &self.account_id)?;

        if let Some(detail) = &self.account_detail {
            template.push_str(&format_tlv("01", detail)?);
        }
        if let Some(bank) = &self.acquiring_bank {
            template.push_str(&format_tlv("02", bank)?);
        }

        Ok(template)
    }

    fn additional_template(&self) -> Result<String, KhqrError> {
        let mut template = String::new();

        for (tag, _, value) in self.additional.fields() {
            if let Some(value) = value {
                template.push_str(&format_tlv(tag, value)?);
            }
        }

        Ok(template)
    }

    fn timestamp_template(&self) -> Result<String, KhqrError> {
        let mut template = String::new();

        if let Some(created) = self.created_at_ms {
            template.push_str(&format_tlv("00", &created.to_string())?);
        }
        if let Some(expires) = self.expires_at_ms {
            template.push_str(&format_tlv("01", &expires.to_string())?);
        }

        Ok(template)
    }
}

fn language_template(language: &AlternateLanguage) -> Result<String, KhqrError> {
    let mut template = format_tlv("00", &language.preference)?;
    template.push_str(&format_tlv("01", &language.name)?);
    template.push_str(&format_tlv("02", &language.city)?);

    Ok(template)
}

/// Collects fields for a [`Khqr`], which validates them on [`KhqrBuilder::build`].
#[derive(Debug, Clone)]
pub struct KhqrBuilder {
    merchant_type: MerchantType,
    account_id: String,
    account_detail: Option<String>,
    acquiring_bank: Option<String>,
    union_pay_merchant: Option<String>,
    merchant_category_code: Option<String>,
    currency: Currency,
    amount: Option<f64>,
    country_code: Option<String>,
    merchant_name: Option<String>,
    merchant_city: Option<String>,
    additional: AdditionalData,
    alternate_language: Option<AlternateLanguage>,
    created_at_ms: Option<u64>,
    expires_at_ms: Option<u64>,
}

impl KhqrBuilder {
    fn new(merchant_type: MerchantType, account_id: String) -> Self {
        Self {
            merchant_type,
            account_id,
            account_detail: None,
            acquiring_bank: None,
            union_pay_merchant: None,
            merchant_category_code: None,
            currency: Currency::Khr,
            amount: None,
            country_code: None,
            merchant_name: None,
            merchant_city: None,
            additional: AdditionalData::default(),
            alternate_language: None,
            created_at_ms: None,
            expires_at_ms: None,
        }
    }

    /// Account information, tag `29` sub-tag `01`.
    pub fn account_information(mut self, value: impl Into<String>) -> Self {
        self.account_detail = Some(value.into());
        self
    }

    /// Merchant ID, tag `30` sub-tag `01`.
    pub fn merchant_id(mut self, value: impl Into<String>) -> Self {
        self.account_detail = Some(value.into());
        self
    }

    /// Acquiring bank, sub-tag `02` of whichever account tag is in use.
    pub fn acquiring_bank(mut self, value: impl Into<String>) -> Self {
        self.acquiring_bank = Some(value.into());
        self
    }

    /// UnionPay merchant account, tag `15`.
    pub fn union_pay_merchant(mut self, value: impl Into<String>) -> Self {
        self.union_pay_merchant = Some(value.into());
        self
    }

    /// Merchant category code. Defaults to `5999`.
    pub fn merchant_category_code(mut self, value: impl Into<String>) -> Self {
        self.merchant_category_code = Some(value.into());
        self
    }

    /// Transaction currency. Defaults to riel.
    pub fn currency(mut self, currency: Currency) -> Self {
        self.currency = currency;
        self
    }

    /// Transaction amount. Setting one makes the payload single use.
    pub fn amount(mut self, amount: f64) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Country code. Defaults to `KH`.
    pub fn country_code(mut self, value: impl Into<String>) -> Self {
        self.country_code = Some(value.into());
        self
    }

    /// Merchant name, at most 25 characters. Required.
    pub fn merchant_name(mut self, value: impl Into<String>) -> Self {
        self.merchant_name = Some(value.into());
        self
    }

    /// Merchant city, at most 15 characters. Required.
    pub fn merchant_city(mut self, value: impl Into<String>) -> Self {
        self.merchant_city = Some(value.into());
        self
    }

    /// Bill number, tag `62` sub-tag `01`.
    pub fn bill_number(mut self, value: impl Into<String>) -> Self {
        self.additional.bill_number = Some(value.into());
        self
    }

    /// Mobile number, tag `62` sub-tag `02`.
    pub fn mobile_number(mut self, value: impl Into<String>) -> Self {
        self.additional.mobile_number = Some(value.into());
        self
    }

    /// Store label, tag `62` sub-tag `03`.
    pub fn store_label(mut self, value: impl Into<String>) -> Self {
        self.additional.store_label = Some(value.into());
        self
    }

    /// Reference label, tag `62` sub-tag `05`.
    pub fn reference_label(mut self, value: impl Into<String>) -> Self {
        self.additional.reference_label = Some(value.into());
        self
    }

    /// Terminal label, tag `62` sub-tag `07`.
    pub fn terminal_label(mut self, value: impl Into<String>) -> Self {
        self.additional.terminal_label = Some(value.into());
        self
    }

    /// Purpose of transaction, tag `62` sub-tag `08`.
    pub fn purpose_of_transaction(mut self, value: impl Into<String>) -> Self {
        self.additional.purpose = Some(value.into());
        self
    }

    /// Name and city in a second language, tag `64`. Khmer is `KM`.
    pub fn alternate_language(
        mut self,
        preference: impl Into<String>,
        name: impl Into<String>,
        city: impl Into<String>,
    ) -> Self {
        self.alternate_language = Some(AlternateLanguage {
            preference: preference.into(),
            name: name.into(),
            city: city.into(),
        });
        self
    }

    /// Creation time in epoch milliseconds, tag `99` sub-tag `00`.
    pub fn created_at_ms(mut self, millis: u64) -> Self {
        self.created_at_ms = Some(millis);
        self
    }

    /// Expiry time in epoch milliseconds, tag `99` sub-tag `01`.
    pub fn expires_at_ms(mut self, millis: u64) -> Self {
        self.expires_at_ms = Some(millis);
        self
    }

    /// Checks every field against the specification.
    pub fn build(self) -> Result<Khqr, KhqrError> {
        if !is_account_id(&self.account_id) {
            return Err(invalid("account id", &self.account_id));
        }

        capped(&self.account_id, "account id", MAX_ACCOUNT)?;

        if let Some(detail) = &self.account_detail {
            capped(detail, "account information", MAX_ACCOUNT)?;
        }
        if let Some(bank) = &self.acquiring_bank {
            capped(bank, "acquiring bank", MAX_ACCOUNT)?;
        }
        if let Some(union_pay) = &self.union_pay_merchant {
            capped(union_pay, "unionpay merchant", MAX_UNION_PAY)?;
        }
        for (_, field, value) in self.additional.fields() {
            if let Some(value) = value {
                capped(value, field, MAX_LABEL)?;
            }
        }
        for (millis, field) in [
            (self.created_at_ms, "creation timestamp"),
            (self.expires_at_ms, "expiration timestamp"),
        ] {
            if let Some(millis) = millis {
                capped(&millis.to_string(), field, MAX_TIMESTAMP)?;
            }
        }

        let merchant_name = required(self.merchant_name, "merchant name")?;
        capped(&merchant_name, "merchant name", MAX_NAME)?;

        let merchant_city = required(self.merchant_city, "merchant city")?;
        capped(&merchant_city, "merchant city", MAX_CITY)?;

        let merchant_category_code = self
            .merchant_category_code
            .unwrap_or_else(|| DEFAULT_CATEGORY_CODE.to_string());
        if merchant_category_code.len() != 4
            || !merchant_category_code.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(invalid("merchant category code", &merchant_category_code));
        }

        let country_code = self
            .country_code
            .unwrap_or_else(|| DEFAULT_COUNTRY_CODE.to_string());
        if country_code.len() != 2 || !country_code.bytes().all(|b| b.is_ascii_uppercase()) {
            return Err(invalid("country code", &country_code));
        }

        if let Some(language) = &self.alternate_language {
            if language.preference.chars().count() != 2 {
                return Err(invalid("language preference", &language.preference));
            }
            capped(&language.name, "alternate merchant name", MAX_NAME)?;
            capped(&language.city, "alternate merchant city", MAX_CITY)?;
        }

        let amount = match self.amount {
            Some(amount) => Some(format_amount(amount, self.currency)?),
            None => None,
        };

        Ok(Khqr {
            merchant_type: self.merchant_type,
            account_id: self.account_id,
            account_detail: self.account_detail,
            acquiring_bank: self.acquiring_bank,
            union_pay_merchant: self.union_pay_merchant,
            merchant_category_code,
            currency: self.currency,
            amount,
            country_code,
            merchant_name,
            merchant_city,
            additional: self.additional,
            alternate_language: self.alternate_language,
            created_at_ms: self.created_at_ms,
            expires_at_ms: self.expires_at_ms,
        })
    }
}

fn invalid(field: &'static str, value: &str) -> KhqrError {
    KhqrError::InvalidField {
        field,
        value: value.to_string(),
    }
}

fn required(value: Option<String>, field: &'static str) -> Result<String, KhqrError> {
    value.ok_or(KhqrError::MissingField { field })
}

fn capped(value: &str, field: &'static str, max: usize) -> Result<(), KhqrError> {
    let chars = value.chars().count();

    if chars > max {
        return Err(KhqrError::FieldTooLong { field, chars, max });
    }

    Ok(())
}

fn is_account_id(account_id: &str) -> bool {
    match account_id.split_once('@') {
        Some((name, bank)) => !name.is_empty() && !bank.is_empty() && !bank.contains('@'),
        None => false,
    }
}

fn format_amount(amount: f64, currency: Currency) -> Result<String, KhqrError> {
    if !amount.is_finite() || amount < 0.0 {
        return Err(invalid("amount", &amount.to_string()));
    }

    let decimals = currency.decimals();
    let text = format!("{amount:.decimals$}");
    capped(&text, "amount", MAX_AMOUNT)?;

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn riel_amounts_carry_no_decimals() {
        assert_eq!(format_amount(500.0, Currency::Khr).unwrap(), "500");
        assert_eq!(format_amount(500.7, Currency::Khr).unwrap(), "501");
    }

    #[test]
    fn dollar_amounts_carry_two() {
        assert_eq!(format_amount(0.1, Currency::Usd).unwrap(), "0.10");
        assert_eq!(format_amount(1234.5, Currency::Usd).unwrap(), "1234.50");
    }

    #[test]
    fn amounts_must_be_finite_and_positive() {
        assert!(format_amount(-1.0, Currency::Khr).is_err());
        assert!(format_amount(f64::NAN, Currency::Khr).is_err());
        assert!(format_amount(f64::INFINITY, Currency::Khr).is_err());
    }

    #[test]
    fn an_amount_too_wide_for_the_field_is_rejected() {
        assert!(format_amount(1e13, Currency::Khr).is_err());
    }

    #[test]
    fn an_account_id_needs_a_name_and_a_bank() {
        assert!(is_account_id("jonhsmith@nbcq"));
        assert!(!is_account_id("jonhsmith"));
        assert!(!is_account_id("@nbcq"));
        assert!(!is_account_id("jonhsmith@"));
        assert!(!is_account_id("a@b@c"));
    }
}
