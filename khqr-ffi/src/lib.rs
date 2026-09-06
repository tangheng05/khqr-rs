//! Kotlin, Swift and Python bindings over `khqr-core`.
//!
//! Encoding only: no async runtime and no HTTP, so the same library works on a
//! phone, a POS terminal and a server.
#![forbid(unsafe_code)]

uniffi::setup_scaffolding!();

/// Anything that goes wrong building or reading a payload.
#[derive(Debug, uniffi::Error)]
pub enum KhqrError {
    /// The payload or one of its fields was rejected.
    Invalid {
        /// What was wrong.
        message: String,
    },
}

impl std::fmt::Display for KhqrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid { message } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for KhqrError {}

impl From<khqr_core::KhqrError> for KhqrError {
    fn from(error: khqr_core::KhqrError) -> Self {
        Self::Invalid {
            message: error.to_string(),
        }
    }
}

/// The currency a payment is denominated in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum Currency {
    /// Cambodian riel, no decimal places.
    Khr,
    /// US dollar, two decimal places.
    Usd,
}

impl From<Currency> for khqr_core::Currency {
    fn from(currency: Currency) -> Self {
        match currency {
            Currency::Khr => Self::Khr,
            Currency::Usd => Self::Usd,
        }
    }
}

/// Whether the account is a person or a business.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum MerchantType {
    /// Account details live in tag `29`.
    Individual,
    /// Account details live in tag `30`.
    Merchant,
}

impl From<khqr_core::MerchantType> for MerchantType {
    fn from(merchant_type: khqr_core::MerchantType) -> Self {
        match merchant_type {
            khqr_core::MerchantType::Individual => Self::Individual,
            khqr_core::MerchantType::Merchant => Self::Merchant,
        }
    }
}

/// One tag-length-value triple.
#[derive(Debug, Clone, uniffi::Record)]
pub struct Tlv {
    pub tag: String,
    pub value: String,
}

/// Everything needed to build a payload. Only the first four are required.
#[derive(Debug, Clone, uniffi::Record)]
pub struct KhqrOptions {
    pub merchant_type: MerchantType,
    pub account_id: String,
    pub merchant_name: String,
    pub merchant_city: String,
    #[uniffi(default = None)]
    pub currency: Option<Currency>,
    #[uniffi(default = None)]
    pub amount: Option<f64>,
    #[uniffi(default = None)]
    pub account_information: Option<String>,
    #[uniffi(default = None)]
    pub merchant_id: Option<String>,
    #[uniffi(default = None)]
    pub acquiring_bank: Option<String>,
    #[uniffi(default = None)]
    pub union_pay_merchant: Option<String>,
    #[uniffi(default = None)]
    pub merchant_category_code: Option<String>,
    #[uniffi(default = None)]
    pub country_code: Option<String>,
    #[uniffi(default = None)]
    pub bill_number: Option<String>,
    #[uniffi(default = None)]
    pub mobile_number: Option<String>,
    #[uniffi(default = None)]
    pub store_label: Option<String>,
    #[uniffi(default = None)]
    pub reference_label: Option<String>,
    #[uniffi(default = None)]
    pub terminal_label: Option<String>,
    #[uniffi(default = None)]
    pub purpose_of_transaction: Option<String>,
    #[uniffi(default = None)]
    pub language_preference: Option<String>,
    #[uniffi(default = None)]
    pub merchant_name_alternate: Option<String>,
    #[uniffi(default = None)]
    pub merchant_city_alternate: Option<String>,
    #[uniffi(default = None)]
    pub created_at_ms: Option<u64>,
    #[uniffi(default = None)]
    pub expires_at_ms: Option<u64>,
}

/// Every field of a payload, with the nested templates flattened.
#[derive(Debug, Clone, uniffi::Record)]
pub struct Decoded {
    pub merchant_type: MerchantType,
    pub payload_format_indicator: String,
    pub point_of_initiation_method: String,
    pub bakong_account_id: String,
    pub merchant_category_code: String,
    pub transaction_currency: String,
    pub country_code: String,
    pub merchant_name: String,
    pub merchant_city: String,
    pub crc: String,
    pub is_dynamic: bool,
    pub account_information: Option<String>,
    pub merchant_id: Option<String>,
    pub acquiring_bank: Option<String>,
    pub union_pay_merchant: Option<String>,
    pub transaction_amount: Option<String>,
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
    pub unknown: Vec<Tlv>,
}

/// Builds a payload, checksum included.
#[uniffi::export]
pub fn generate(options: KhqrOptions) -> Result<String, KhqrError> {
    let mut builder = match options.merchant_type {
        MerchantType::Individual => khqr_core::Khqr::individual(&options.account_id),
        MerchantType::Merchant => khqr_core::Khqr::merchant(&options.account_id),
    };

    builder = builder
        .merchant_name(&options.merchant_name)
        .merchant_city(&options.merchant_city)
        .currency(options.currency.unwrap_or(Currency::Khr).into());

    if let Some(amount) = options.amount {
        builder = builder.amount(amount);
    }
    // Both write sub-tag 01 of the account template, so only one can be set.
    if options.account_information.is_some() && options.merchant_id.is_some() {
        return Err(KhqrError::Invalid {
            message: "account_information and merchant_id share one field, so set only one"
                .to_string(),
        });
    }
    if let Some(value) = &options.account_information {
        builder = builder.account_information(value);
    }
    if let Some(value) = &options.merchant_id {
        builder = builder.merchant_id(value);
    }
    if let Some(value) = &options.acquiring_bank {
        builder = builder.acquiring_bank(value);
    }
    if let Some(value) = &options.union_pay_merchant {
        builder = builder.union_pay_merchant(value);
    }
    if let Some(value) = &options.merchant_category_code {
        builder = builder.merchant_category_code(value);
    }
    if let Some(value) = &options.country_code {
        builder = builder.country_code(value);
    }
    if let Some(value) = &options.bill_number {
        builder = builder.bill_number(value);
    }
    if let Some(value) = &options.mobile_number {
        builder = builder.mobile_number(value);
    }
    if let Some(value) = &options.store_label {
        builder = builder.store_label(value);
    }
    if let Some(value) = &options.reference_label {
        builder = builder.reference_label(value);
    }
    if let Some(value) = &options.terminal_label {
        builder = builder.terminal_label(value);
    }
    if let Some(value) = &options.purpose_of_transaction {
        builder = builder.purpose_of_transaction(value);
    }
    match (
        &options.language_preference,
        &options.merchant_name_alternate,
        &options.merchant_city_alternate,
    ) {
        (Some(preference), Some(name), Some(city)) => {
            builder = builder.alternate_language(preference, name, city);
        }
        (None, None, None) => {}
        // Dropping a partly filled template silently would lose a merchant's
        // Khmer name with no sign that anything went wrong.
        _ => {
            return Err(KhqrError::Invalid {
                message: "the alternate language needs preference, name and city together"
                    .to_string(),
            })
        }
    }
    if let Some(millis) = options.created_at_ms {
        builder = builder.created_at_ms(millis);
    }
    if let Some(millis) = options.expires_at_ms {
        builder = builder.expires_at_ms(millis);
    }

    Ok(builder.build()?.to_qr_string()?)
}

/// Reads a payload back. Fails if the checksum does not match.
#[uniffi::export]
pub fn decode(qr: String) -> Result<Decoded, KhqrError> {
    let decoded = khqr_core::decode(&qr)?;
    let unknown = decoded
        .unknown
        .iter()
        .chain(&decoded.additional_unknown)
        .map(|field| Tlv {
            tag: field.tag.clone(),
            value: field.value.clone(),
        })
        .collect();

    Ok(Decoded {
        merchant_type: decoded.merchant_type.into(),
        payload_format_indicator: decoded.payload_format_indicator,
        point_of_initiation_method: decoded.point_of_initiation_method,
        bakong_account_id: decoded.bakong_account_id,
        merchant_category_code: decoded.merchant_category_code,
        transaction_currency: decoded.transaction_currency,
        country_code: decoded.country_code,
        merchant_name: decoded.merchant_name,
        merchant_city: decoded.merchant_city,
        crc: decoded.crc,
        is_dynamic: decoded.transaction_amount.is_some(),
        account_information: decoded.account_information,
        merchant_id: decoded.merchant_id,
        acquiring_bank: decoded.acquiring_bank,
        union_pay_merchant: decoded.union_pay_merchant,
        transaction_amount: decoded.transaction_amount,
        bill_number: decoded.bill_number,
        mobile_number: decoded.mobile_number,
        store_label: decoded.store_label,
        reference_label: decoded.reference_label,
        terminal_label: decoded.terminal_label,
        purpose_of_transaction: decoded.purpose_of_transaction,
        language_preference: decoded.language_preference,
        merchant_name_alternate: decoded.merchant_name_alternate,
        merchant_city_alternate: decoded.merchant_city_alternate,
        created_at_ms: decoded.created_at_ms,
        expires_at_ms: decoded.expires_at_ms,
        unknown,
    })
}

/// Whether the payload's checksum matches its contents.
#[uniffi::export]
pub fn verify(qr: String) -> bool {
    khqr_core::verify_crc(&qr)
}

/// The MD5 handle used to poll for payment.
#[uniffi::export]
pub fn md5(qr: String) -> String {
    khqr_core::md5(&qr)
}

/// Renders the payload as PNG bytes, at least `size` pixels wide.
#[uniffi::export]
pub fn to_png(qr: String, size: u32) -> Result<Vec<u8>, KhqrError> {
    Ok(khqr_core::to_png(&qr, size)?)
}

/// Renders the payload as an SVG.
#[uniffi::export]
pub fn to_svg(qr: String) -> Result<String, KhqrError> {
    Ok(khqr_core::to_svg(&qr)?)
}

/// Renders the payload as a PNG data URI, ready for an image widget.
#[uniffi::export]
pub fn to_data_uri(qr: String, size: u32) -> Result<String, KhqrError> {
    Ok(khqr_core::to_base64_uri(&qr, size)?)
}
