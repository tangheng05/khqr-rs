//! Browser and Node bindings over `khqr-core`.
//!
//! Encoding only: no async runtime, no HTTP, so a KHQR payload can be built
//! and rendered client side with no server round trip.
#![forbid(unsafe_code)]

use khqr_core::{Currency, KhqrBuilder, KhqrError};
use wasm_bindgen::prelude::*;

fn to_js(error: KhqrError) -> JsValue {
    JsValue::from_str(&error.to_string())
}

/// Builds a KHQR payload. Set what you need, then call `build`.
#[wasm_bindgen]
pub struct Khqr {
    builder: Option<KhqrBuilder>,
}

#[wasm_bindgen]
impl Khqr {
    /// A payload for a personal account, written to tag `29`.
    pub fn individual(account_id: &str) -> Self {
        Self {
            builder: Some(khqr_core::Khqr::individual(account_id)),
        }
    }

    /// A payload for a business account, written to tag `30`.
    pub fn merchant(account_id: &str) -> Self {
        Self {
            builder: Some(khqr_core::Khqr::merchant(account_id)),
        }
    }

    #[wasm_bindgen(js_name = merchantName)]
    pub fn merchant_name(&mut self, value: &str) {
        self.set(|builder| builder.merchant_name(value));
    }

    #[wasm_bindgen(js_name = merchantCity)]
    pub fn merchant_city(&mut self, value: &str) {
        self.set(|builder| builder.merchant_city(value));
    }

    /// `KHR` or `USD`. Anything else is ignored and riel is kept.
    pub fn currency(&mut self, value: &str) {
        let currency = match value.to_ascii_uppercase().as_str() {
            "USD" | "840" => Currency::Usd,
            _ => Currency::Khr,
        };

        self.set(|builder| builder.currency(currency));
    }

    /// Setting an amount makes the payload single use.
    pub fn amount(&mut self, value: f64) {
        self.set(|builder| builder.amount(value));
    }

    #[wasm_bindgen(js_name = merchantId)]
    pub fn merchant_id(&mut self, value: &str) {
        self.set(|builder| builder.merchant_id(value));
    }

    #[wasm_bindgen(js_name = accountInformation)]
    pub fn account_information(&mut self, value: &str) {
        self.set(|builder| builder.account_information(value));
    }

    #[wasm_bindgen(js_name = acquiringBank)]
    pub fn acquiring_bank(&mut self, value: &str) {
        self.set(|builder| builder.acquiring_bank(value));
    }

    #[wasm_bindgen(js_name = unionPayMerchant)]
    pub fn union_pay_merchant(&mut self, value: &str) {
        self.set(|builder| builder.union_pay_merchant(value));
    }

    #[wasm_bindgen(js_name = merchantCategoryCode)]
    pub fn merchant_category_code(&mut self, value: &str) {
        self.set(|builder| builder.merchant_category_code(value));
    }

    #[wasm_bindgen(js_name = countryCode)]
    pub fn country_code(&mut self, value: &str) {
        self.set(|builder| builder.country_code(value));
    }

    #[wasm_bindgen(js_name = billNumber)]
    pub fn bill_number(&mut self, value: &str) {
        self.set(|builder| builder.bill_number(value));
    }

    #[wasm_bindgen(js_name = mobileNumber)]
    pub fn mobile_number(&mut self, value: &str) {
        self.set(|builder| builder.mobile_number(value));
    }

    #[wasm_bindgen(js_name = storeLabel)]
    pub fn store_label(&mut self, value: &str) {
        self.set(|builder| builder.store_label(value));
    }

    #[wasm_bindgen(js_name = referenceLabel)]
    pub fn reference_label(&mut self, value: &str) {
        self.set(|builder| builder.reference_label(value));
    }

    #[wasm_bindgen(js_name = terminalLabel)]
    pub fn terminal_label(&mut self, value: &str) {
        self.set(|builder| builder.terminal_label(value));
    }

    #[wasm_bindgen(js_name = purposeOfTransaction)]
    pub fn purpose_of_transaction(&mut self, value: &str) {
        self.set(|builder| builder.purpose_of_transaction(value));
    }

    /// Name and city in a second language, tag `64`. Khmer is `KM`.
    #[wasm_bindgen(js_name = alternateLanguage)]
    pub fn alternate_language(&mut self, preference: &str, name: &str, city: &str) {
        self.set(|builder| builder.alternate_language(preference, name, city));
    }

    #[wasm_bindgen(js_name = createdAtMs)]
    pub fn created_at_ms(&mut self, millis: f64) {
        self.set(|builder| builder.created_at_ms(millis as u64));
    }

    #[wasm_bindgen(js_name = expiresAtMs)]
    pub fn expires_at_ms(&mut self, millis: f64) {
        self.set(|builder| builder.expires_at_ms(millis as u64));
    }

    /// Validates everything and returns the payload string.
    pub fn build(&mut self) -> Result<String, JsValue> {
        let builder = self
            .builder
            .take()
            .ok_or_else(|| JsValue::from_str("this builder has already been used"))?;

        builder
            .build()
            .map_err(to_js)?
            .to_qr_string()
            .map_err(to_js)
    }

    fn set(&mut self, apply: impl FnOnce(KhqrBuilder) -> KhqrBuilder) {
        if let Some(builder) = self.builder.take() {
            self.builder = Some(apply(builder));
        }
    }
}

/// Whether the payload's checksum matches its contents.
#[wasm_bindgen]
pub fn verify(qr: &str) -> bool {
    khqr_core::verify_crc(qr)
}

/// The MD5 handle used to poll for payment.
#[wasm_bindgen]
pub fn md5(qr: &str) -> String {
    khqr_core::md5(qr)
}

/// Renders the payload as an SVG.
#[wasm_bindgen(js_name = toSvg)]
pub fn to_svg(qr: &str) -> Result<String, JsValue> {
    khqr_core::to_svg(qr).map_err(to_js)
}

/// Renders the payload as PNG bytes, at least `size` pixels wide.
#[wasm_bindgen(js_name = toPng)]
pub fn to_png(qr: &str, size: u32) -> Result<Vec<u8>, JsValue> {
    khqr_core::to_png(qr, size).map_err(to_js)
}

/// Renders the payload as a PNG data URI, ready for an `img` tag.
#[wasm_bindgen(js_name = toDataUri)]
pub fn to_data_uri(qr: &str, size: u32) -> Result<String, JsValue> {
    khqr_core::to_base64_uri(qr, size).map_err(to_js)
}

/// Reads a payload back. Throws if the checksum does not match.
#[wasm_bindgen]
pub fn decode(qr: &str) -> Result<Decoded, JsValue> {
    khqr_core::decode(qr)
        .map(|decoded| Decoded { inner: decoded })
        .map_err(to_js)
}

/// The fields of a decoded payload.
#[wasm_bindgen]
pub struct Decoded {
    inner: khqr_core::DecodedKhqr,
}

#[wasm_bindgen]
impl Decoded {
    #[wasm_bindgen(getter, js_name = payloadFormatIndicator)]
    pub fn payload_format_indicator(&self) -> String {
        self.inner.payload_format_indicator.clone()
    }

    #[wasm_bindgen(getter, js_name = pointOfInitiationMethod)]
    pub fn point_of_initiation_method(&self) -> String {
        self.inner.point_of_initiation_method.clone()
    }

    #[wasm_bindgen(getter, js_name = bakongAccountId)]
    pub fn bakong_account_id(&self) -> String {
        self.inner.bakong_account_id.clone()
    }

    #[wasm_bindgen(getter, js_name = merchantCategoryCode)]
    pub fn merchant_category_code(&self) -> String {
        self.inner.merchant_category_code.clone()
    }

    #[wasm_bindgen(getter, js_name = transactionCurrency)]
    pub fn transaction_currency(&self) -> String {
        self.inner.transaction_currency.clone()
    }

    #[wasm_bindgen(getter, js_name = countryCode)]
    pub fn country_code(&self) -> String {
        self.inner.country_code.clone()
    }

    #[wasm_bindgen(getter, js_name = merchantName)]
    pub fn merchant_name(&self) -> String {
        self.inner.merchant_name.clone()
    }

    #[wasm_bindgen(getter, js_name = merchantCity)]
    pub fn merchant_city(&self) -> String {
        self.inner.merchant_city.clone()
    }

    #[wasm_bindgen(getter, js_name = crc)]
    pub fn crc(&self) -> String {
        self.inner.crc.clone()
    }

    #[wasm_bindgen(getter, js_name = accountInformation)]
    pub fn account_information(&self) -> Option<String> {
        self.inner.account_information.clone()
    }

    #[wasm_bindgen(getter, js_name = merchantId)]
    pub fn merchant_id(&self) -> Option<String> {
        self.inner.merchant_id.clone()
    }

    #[wasm_bindgen(getter, js_name = acquiringBank)]
    pub fn acquiring_bank(&self) -> Option<String> {
        self.inner.acquiring_bank.clone()
    }

    #[wasm_bindgen(getter, js_name = unionPayMerchant)]
    pub fn union_pay_merchant(&self) -> Option<String> {
        self.inner.union_pay_merchant.clone()
    }

    #[wasm_bindgen(getter, js_name = transactionAmount)]
    pub fn transaction_amount(&self) -> Option<String> {
        self.inner.transaction_amount.clone()
    }

    #[wasm_bindgen(getter, js_name = billNumber)]
    pub fn bill_number(&self) -> Option<String> {
        self.inner.bill_number.clone()
    }

    #[wasm_bindgen(getter, js_name = mobileNumber)]
    pub fn mobile_number(&self) -> Option<String> {
        self.inner.mobile_number.clone()
    }

    #[wasm_bindgen(getter, js_name = storeLabel)]
    pub fn store_label(&self) -> Option<String> {
        self.inner.store_label.clone()
    }

    #[wasm_bindgen(getter, js_name = referenceLabel)]
    pub fn reference_label(&self) -> Option<String> {
        self.inner.reference_label.clone()
    }

    #[wasm_bindgen(getter, js_name = terminalLabel)]
    pub fn terminal_label(&self) -> Option<String> {
        self.inner.terminal_label.clone()
    }

    #[wasm_bindgen(getter, js_name = purposeOfTransaction)]
    pub fn purpose_of_transaction(&self) -> Option<String> {
        self.inner.purpose_of_transaction.clone()
    }

    #[wasm_bindgen(getter, js_name = languagePreference)]
    pub fn language_preference(&self) -> Option<String> {
        self.inner.language_preference.clone()
    }

    #[wasm_bindgen(getter, js_name = merchantNameAlternateLanguage)]
    pub fn merchant_name_alternate(&self) -> Option<String> {
        self.inner.merchant_name_alternate.clone()
    }

    #[wasm_bindgen(getter, js_name = merchantCityAlternateLanguage)]
    pub fn merchant_city_alternate(&self) -> Option<String> {
        self.inner.merchant_city_alternate.clone()
    }

    /// `"29"` for an individual, `"30"` for a merchant.
    #[wasm_bindgen(getter, js_name = merchantType)]
    pub fn merchant_type(&self) -> String {
        self.inner.merchant_type.tag().to_string()
    }

    #[wasm_bindgen(getter, js_name = createdAtMs)]
    pub fn created_at_ms(&self) -> Option<f64> {
        self.inner.created_at_ms.map(|millis| millis as f64)
    }

    #[wasm_bindgen(getter, js_name = expiresAtMs)]
    pub fn expires_at_ms(&self) -> Option<f64> {
        self.inner.expires_at_ms.map(|millis| millis as f64)
    }

    #[wasm_bindgen(getter, js_name = isDynamic)]
    pub fn is_dynamic(&self) -> bool {
        self.inner.is_dynamic()
    }

    /// Tags this library does not recognise, as `tag=value` strings.
    #[wasm_bindgen(getter, js_name = unknownTags)]
    pub fn unknown_tags(&self) -> Vec<String> {
        self.inner
            .unknown
            .iter()
            .chain(&self.inner.additional_unknown)
            .map(|field| format!("{}={}", field.tag, field.value))
            .collect()
    }
}
