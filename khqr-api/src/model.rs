use serde::{Deserialize, Serialize};

/// Which Bakong deployment to talk to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    /// The live service.
    Production,
    /// The sandbox, for testing against issued sandbox tokens.
    Sandbox,
}

impl Environment {
    /// The base URL requests are sent to.
    pub fn base_url(self) -> &'static str {
        match self {
            Self::Production => "https://api-bakong.nbc.gov.kh",
            Self::Sandbox => "https://sit-api-bakong.nbc.gov.kh",
        }
    }
}

/// What Bakong knows about one payment.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub hash: String,
    #[serde(default)]
    pub from_account_id: Option<String>,
    #[serde(default)]
    pub to_account_id: Option<String>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub amount: Option<f64>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub created_date_ms: Option<i64>,
    #[serde(default)]
    pub acknowledged_date_ms: Option<i64>,
    #[serde(default)]
    pub external_ref: Option<String>,
}

/// The outcome of asking about one payment.
#[derive(Debug, Clone, PartialEq)]
pub enum TxStatus {
    /// Bakong has the transaction.
    Paid(Box<Transaction>),
    /// Nothing yet. Keep polling, but back off.
    NotFound,
    /// The QR carries no amount, so Bakong cannot track it.
    StaticQr,
}

impl TxStatus {
    /// The transaction, if there is one.
    pub fn transaction(&self) -> Option<&Transaction> {
        match self {
            Self::Paid(transaction) => Some(transaction),
            _ => None,
        }
    }

    /// Whether polling again could still change the answer.
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::NotFound)
    }
}

/// The calling app, shown to the user by the Bakong app on a deeplink.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInfo {
    pub app_name: String,
    pub app_icon_url: String,
    pub app_deep_link_callback: String,
}

impl SourceInfo {
    /// All three fields are required by the endpoint.
    pub fn new(
        app_name: impl Into<String>,
        app_icon_url: impl Into<String>,
        app_deep_link_callback: impl Into<String>,
    ) -> Self {
        Self {
            app_name: app_name.into(),
            app_icon_url: app_icon_url.into(),
            app_deep_link_callback: app_deep_link_callback.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", bound(deserialize = "T: Deserialize<'de>"))]
pub(crate) struct Envelope<T> {
    pub response_code: i64,
    #[serde(default)]
    pub response_message: Option<String>,
    #[serde(default)]
    pub error_code: Option<i64>,
    #[serde(default)]
    pub data: Option<T>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Token {
    pub token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Deeplink {
    pub short_link: String,
}
